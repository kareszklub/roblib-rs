use crate::RemoteRobotTransport;
use actix_http::ws::Frame;
use actix_rt::{spawn, task::JoinHandle};
use actix_web::web::Bytes;
use anyhow::{anyhow, Result};
use awc::{ws::Message, Client};
use futures::{
    channel::mpsc::{self, UnboundedReceiver, UnboundedSender},
    executor::block_on,
    SinkExt,
};
use futures_util::{lock::Mutex, stream::StreamExt};
use roblib::cmd::Cmd;

pub struct RobotWS {
    tx: Mutex<UnboundedSender<Message>>,
    rx: Mutex<UnboundedReceiver<String>>,
    sender: JoinHandle<()>,
    receiver: JoinHandle<()>,
}

impl RobotWS {
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
            tx: tx_t.into(),
            rx: rx_r.into(),
            sender,
            receiver,
        })
    }

    /// Send a raw command.
    /// You probably don't need this.
    pub async fn send(&self, cmd: &str) -> Result<String> {
        self.tx.lock().await.send(Message::Text(cmd.into())).await?;
        self.rx
            .lock()
            .await
            .next()
            .await
            .ok_or_else(|| anyhow!("no message received"))
    }

    pub async fn disconnect(&self) -> Result<()> {
        debug!("disconnecting");

        self.tx.lock().await.send(Message::Close(None)).await?;
        self.sender.abort();
        self.receiver.abort();

        info!("disconnected");
        Ok(())
    }
}

impl RemoteRobotTransport for RobotWS {
    fn cmd(&self, cmd: Cmd) -> Result<String> {
        block_on(async {
            let s = cmd.to_string();
            debug!("S: {}", &s);
            let r = self.send(&s).await?;
            debug!("R: {}", &r);
            Ok(r)
        })
    }
}
