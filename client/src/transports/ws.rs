use actix_http::ws::Frame;
use actix_rt::{spawn, Runtime};
use actix_web::web::Bytes;
use anyhow::Result;
use awc::{ws::Message, Client};
use futures::{
    channel::mpsc::{self, UnboundedReceiver, UnboundedSender},
    SinkExt,
};
use futures_util::{lock::Mutex, stream::StreamExt};
use roblib::{
    cmd::{Command, Concrete},
    Readable, Writable,
};
use roblib_parsing::SEPARATOR;

struct WSBase {
    tx: Mutex<UnboundedSender<Message>>,
    rx: Mutex<UnboundedReceiver<WSResponse>>,
}

enum WSResponse {
    Binary(Bytes),
    Text(String),
}

impl WSBase {
    pub async fn create(addr: &str) -> Result<Self> {
        let (tx, rx) = Self::create_inner(addr).await?;
        Ok(Self { tx, rx })
    }

    async fn create_inner(
        addr: &str,
    ) -> Result<(
        Mutex<UnboundedSender<Message>>,
        Mutex<UnboundedReceiver<WSResponse>>,
    )> {
        let ws = match Client::new().ws(addr).connect().await {
            Ok((_, ws)) => ws,
            Err(e) => return Err(anyhow::anyhow!("Websocket failed to connect because: {e}")),
        };

        // twx: websocket sender, rwx: websocket receiver (tasks)
        let (mut twx, mut rwx) = ws.split();

        // tx_t: send message to server, rx_t: receive messages to be sent (sender task)
        let (tx_t, mut rx_t) = mpsc::unbounded::<Message>();

        // tx_r: send messages to be received (receiver task), rx_r: receive messages
        let (mut tx_r, rx_r) = mpsc::unbounded::<WSResponse>();

        let tx_ref = tx_t.clone();

        // sender task
        spawn(async move {
            while let Some(msg) = rx_t.next().await {
                twx.send(msg).await.unwrap();
            }
        });

        // receiver task
        spawn(async move {
            let mut tx = tx_ref;
            while let Some(Ok(msg)) = rwx.next().await {
                match msg {
                    Frame::Text(b) => tx_r
                        .send(WSResponse::Text(String::from_utf8(b.to_vec()).unwrap()))
                        .await
                        .unwrap(),

                    Frame::Binary(b) => tx_r.send(WSResponse::Binary(b)).await.unwrap(),

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

        Ok((tx_t.into(), rx_r.into()))
    }

    async fn send(&self, cmd: Concrete) -> Result<WSResponse> {
        let mut buf = vec![];
        cmd.write_binary(&mut buf)?;

        self.tx
            .lock()
            .await
            .send(Message::Binary(buf.into()))
            .await?;

        Ok(if cmd.has_return() {
            let mut rec = self.rx.lock().await;

            rec.next()
                .await
                .ok_or(anyhow::Error::msg("Didn't recieve response"))?
        } else {
            unsafe { std::mem::zeroed() }
        })
    }
}

pub struct Ws {
    ws: WSBase,
    rt: Runtime,
}

impl Ws {
    pub fn connect(addr: &str) -> Result<Self> {
        let rt = Runtime::new()?;
        let ws = rt.block_on(WSBase::create(addr))?;
        Ok(Self { ws, rt })
    }
}

impl super::Transport for Ws {
    fn cmd<C: Command>(&self, cmd: C) -> Result<C::Return>
    where
        C::Return: Readable,
    {
        let res = self.rt.block_on(self.ws.send(cmd.into()))?;
        match res {
            WSResponse::Text(t) => Readable::parse_text(&mut t.split(SEPARATOR)),
            WSResponse::Binary(b) => Readable::parse_binary(&mut std::io::Cursor::new(b)),
        }
    }
}

#[cfg(feature = "async")]
pub struct WsAsync(std::pin::Pin<Box<WSBase>>);

#[cfg(feature = "async")]
impl WsAsync {
    pub async fn connect(addr: &str) -> Result<Self> {
        Ok(Self(Box::pin(WSBase::create(addr).await?)))
    }
}
#[cfg(feature = "async")]
#[cfg_attr(feature = "async", async_trait::async_trait)]
impl super::TransportAsync for WsAsync {
    async fn cmd_async<C: Command + Send>(&self, cmd: C) -> Result<C::Return>
    where
        C::Return: Readable,
    {
        let res = self.0.send(cmd.into()).await?;
        match res {
            WSResponse::Text(t) => Readable::parse_text(&mut t.split(SEPARATOR)),
            WSResponse::Binary(b) => Readable::parse_binary(&mut std::io::Cursor::new(b)),
        }
    }
}
