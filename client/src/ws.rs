use crate::RemoteRobotTransport;
use actix_http::ws::Frame;
use actix_rt::{spawn, task::JoinHandle, Runtime};
use actix_web::web::Bytes;
use anyhow::{anyhow, Result};
use awc::{ws::Message, Client};
use futures::{
    channel::mpsc::{self, UnboundedReceiver, UnboundedSender},
    SinkExt,
};
use futures_util::{lock::Mutex, stream::StreamExt};
use roblib::cmd::{
    parsing::{Readable, Writable},
    Command, SEPARATOR,
};

pub struct RobotWS {
    runtime: Runtime,
    tx: Mutex<UnboundedSender<Message>>,
    rx: Mutex<UnboundedReceiver<String>>,
    sender: JoinHandle<()>,
    receiver: JoinHandle<()>,
}

impl Drop for RobotWS {
    fn drop(&mut self) {
        self.runtime.block_on(self.disconnect()).unwrap()
    }
}

impl RobotWS {
    pub fn create(addr: &str) -> Result<Self> {
        let runtime = Runtime::new()?;
        let (tx, rx, sender, receiver) = runtime.block_on(Self::create_inner(addr))?;

        Ok(Self {
            runtime,
            receiver,
            sender,
            tx,
            rx,
        })
    }

    async fn create_inner(
        addr: &str,
    ) -> Result<(
        Mutex<UnboundedSender<Message>>,
        Mutex<UnboundedReceiver<String>>,
        JoinHandle<()>,
        JoinHandle<()>,
    )> {
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
                twx.send(msg).await.unwrap();
            }
        });

        // receiver task
        let receiver = spawn(async move {
            let mut tx = tx_ref;
            while let Some(Ok(msg)) = rwx.next().await {
                match msg {
                    Frame::Text(b) => tx_r
                        .send(String::from_utf8(b.to_vec()).unwrap())
                        .await
                        .unwrap(),

                    Frame::Binary(_) => error!("received binary frame"),

                    Frame::Continuation(_) => error!("received continuation frame"),

                    Frame::Ping(_) => {
                        tx.send(Message::Pong(Bytes::new())).await.unwrap();
                        trace!("ping");
                    }

                    Frame::Pong(_) => trace!("pong"),

                    Frame::Close(reason) => {
                        tx.close().await.unwrap();
                        error!("socket closed: {reason:?}");
                        break;
                    }
                }
            }
        });

        Ok((tx_t.into(), rx_r.into(), sender, receiver))
    }

    async fn send<C: Command>(&self, cmd: &C) -> Result<C::Return>
    where
        C: Writable,
        C::Return: Readable + Sized,
    {
        let mut s = C::PREFIX.to_string();
        cmd.write_str(&mut |r| {
            s.push(SEPARATOR);
            s.push_str(r);
        })?;

        self.tx.lock().await.send(Message::Text(s.into())).await?;

        Ok(if std::mem::size_of::<C::Return>() != 0 {
            let bs = self
                .rx
                .lock()
                .await
                .next()
                .await
                .ok_or(anyhow::Error::msg("Didn't recieve response"))?;

            Readable::parse_str(&mut bs.split(SEPARATOR))?
        } else {
            unsafe { std::mem::zeroed() }
        })
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
    fn cmd<C: Command>(&self, cmd: C) -> Result<C::Return>
    where
        C::Return: Readable,
    {
        self.runtime.block_on(self.send(&cmd))
    }
}
