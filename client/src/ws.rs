use actix_http::ws::Frame;
use actix_rt::{spawn, task::JoinHandle};
use actix_web::web::Bytes;
use anyhow::{anyhow, Result};
use awc::{ws::Message, Client};
use futures::{
    channel::mpsc::{self, UnboundedReceiver, UnboundedSender},
    SinkExt,
};
use futures_util::stream::StreamExt;
use roblib::cmd::{get_time, parse_sensor_data, Cmd, SensorData};

pub struct Robot {
    tx: UnboundedSender<Message>,
    rx: UnboundedReceiver<String>,
    sender: JoinHandle<()>,
    receiver: JoinHandle<()>,
}

impl Robot {
    pub async fn connect(addr: &str) -> Result<Self> {
        let (_, ws) = match Client::new().ws(addr).connect().await {
            Ok(x) => x,
            Err(_) => return Err(anyhow!("failed to connect")),
        };

        // twx: websocket sender, rwx: websocket receiver (tasks)
        let (mut twx, mut rwx) = ws.split();
        // tx_t: send message to server, rx_t: receive messages to be sent (sender task)
        let (tx_t, mut rx_t) = mpsc::unbounded::<Message>();
        // tx_r: send messages to be received (receiver task), rx_r: receive messages
        let (mut tx_r, rx_r) = mpsc::unbounded::<String>();

        let tx_ref = tx_t.clone();

        // sender task
        let sender = spawn(async move {
            while let Some(msg) = rx_t.next().await {
                if (twx.send(msg).await).is_err() {
                    break;
                }
            }
        });

        // receiver task
        let receiver = spawn(async move {
            let mut tx = tx_ref;
            while let Some(Ok(msg)) = rwx.next().await {
                match msg {
                    Frame::Text(b) => {
                        tx_r.send(String::from_utf8(b.to_vec()).unwrap())
                            .await
                            .unwrap();
                    }
                    Frame::Binary(_) => {
                        error!("received binary frame");
                    }
                    Frame::Continuation(_) => {
                        error!("received continuation frame");
                    }
                    Frame::Ping(_) => {
                        tx.send(Message::Pong(Bytes::new())).await.unwrap();
                        trace!("ping");
                    }
                    Frame::Pong(_) => {
                        trace!("pong");
                    }
                    Frame::Close(reason) => {
                        // self.ws.close().await?;
                        error!("socket closed: {reason:?}");
                    }
                }
            }
        });

        Ok(Self {
            tx: tx_t,
            rx: rx_r,
            sender,
            receiver,
        })
    }

    /// Send a raw command.
    /// You probably don't need this.
    pub async fn send(&mut self, cmd: &str) -> Result<String> {
        self.tx.send(Message::Text(cmd.into())).await?;
        self.rx
            .next()
            .await
            .ok_or_else(|| anyhow!("no message received"))
    }

    pub async fn disconnect(&mut self) -> Result<()> {
        debug!("disconnecting");

        self.tx.send(Message::Close(None)).await?;
        self.sender.abort();
        self.receiver.abort();

        info!("disconnected");
        Ok(())
    }

    pub async fn cmd(&mut self, cmd: Cmd) -> Result<String> {
        let s = cmd.to_string();
        debug!("S: {}", &s);
        let r = self.send(&s).await?;
        debug!("R: {}", &r);
        Ok(r)
    }

    #[cfg(feature = "roland")]
    pub async fn get_sensor_data(&mut self) -> Result<SensorData> {
        parse_sensor_data(&self.cmd(Cmd::TrackSensor).await?)
    }

    pub async fn measure_latency(&mut self) -> Result<f64> {
        let start = get_time()?;
        self.cmd(Cmd::GetTime).await?;
        Ok(get_time()? - start)
    }
}
