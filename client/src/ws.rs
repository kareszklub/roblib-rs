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
use roblib_shared::cmd::{get_time, SensorData};

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

    pub async fn move_robot(&mut self, left: i8, right: i8) -> Result<String> {
        let s = format!("m {} {}", left, right);
        debug!("S: {}", s);
        self.send(&s).await
    }
    pub async fn stop_robot(&mut self) -> Result<String> {
        debug!("S: s");
        self.send("s").await
    }
    pub async fn led(&mut self, (r, g, b): (bool, bool, bool)) -> Result<String> {
        let s = format!("l {} {} {}", r as i8, g as i8, b as i8);
        debug!("S: {}", s);
        self.send(&s).await
    }
    pub async fn servo_absolute(&mut self, angle: f32) -> Result<String> {
        let s = format!("v {}", angle);
        debug!("S: {}", s);
        self.send(&s).await
    }
    pub async fn buzzer(&mut self, freq: u16) -> Result<String> {
        let s = format!("b {}", freq);
        debug!("S: {}", s);
        self.send(&s).await
    }
    pub async fn get_sensor_data(&mut self) -> Result<SensorData> {
        debug!("S: t");
        let d = self
            .send("t")
            .await?
            .split(',')
            .map(|s| s.parse::<i32>().unwrap())
            .collect::<Vec<_>>()
            .try_into()
            .unwrap_or_else(|v: Vec<_>| {
                panic!("Expected a Vec of length {} but it was {}", 4, v.len())
            });
        debug!("R {:?}", d);
        Ok(d)
    }
    pub async fn measure_latency(&mut self) -> Result<f64> {
        let start = get_time();
        debug!("S: ");
        let r = self.send("z").await?;
        debug!("R {}", &r);
        Ok(get_time() - start)
    }
}
