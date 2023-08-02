use super::{SubscribableAsync, TransportAsync};
use anyhow::Result;
use async_trait::async_trait;
use futures::{executor::block_on, SinkExt, TryStreamExt};
use roblib::{
    cmd::{self, has_return},
    event::{ConcreteType, Event},
    text_format,
};
use serde::Deserialize;
use std::{collections::HashMap, io::Cursor, sync::Arc};
use tokio::{
    net::TcpStream,
    sync::{
        broadcast,
        mpsc::{self, unbounded_channel, UnboundedReceiver, UnboundedSender},
        Mutex,
    },
    task::JoinHandle,
};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

type WsConn = WebSocketStream<MaybeTlsStream<TcpStream>>;
type D =
    bincode::Deserializer<bincode::de::read::IoReader<Cursor<Vec<u8>>>, bincode::DefaultOptions>;
type Handler = mpsc::Sender<D>;

#[derive(Default)]
struct WsInner {
    handlers: Mutex<HashMap<u32, Handler>>,
    events: Mutex<HashMap<roblib::event::ConcreteType, u32>>,
}
pub struct Ws {
    inner: Arc<WsInner>,
    handle: Option<JoinHandle<Result<()>>>,
    id: Mutex<u32>,
    sender: UnboundedSender<Message>,
}

enum Action {
    Recv(Message),
    Send(Message),
}
impl Ws {
    pub async fn connect(addr: &str) -> Result<Self> {
        let url = format!("ws://{addr}/ws");
        let (ws, _) = connect_async(url).await?;
        let inner = Arc::new(WsInner::default());

        let (tx, rx) = unbounded_channel();
        let handle = tokio::spawn(Self::worker(ws, rx, inner.clone()));
        Ok(Self {
            inner,
            handle: Some(handle),
            id: super::ID_START.into(),
            sender: tx,
        })
    }

    async fn worker(
        mut ws: WsConn,
        mut rx: UnboundedReceiver<Message>,
        inner: Arc<WsInner>,
    ) -> Result<()> {
        let bin = bincode::options();
        loop {
            let action = tokio::select! {
                Ok(Some(msg)) = ws.try_next() => Action::Recv(msg),
                Some(msg) = rx.recv() => Action::Send(msg)
            };
            match action {
                Action::Recv(msg) => match msg {
                    Message::Text(s) => {
                        let _de = text_format::de::new_deserializer(&s);
                        todo!()
                    }
                    Message::Binary(b) => {
                        let mut c = Cursor::new(b);
                        let id: u32 = bincode::Options::deserialize_from(bin, &mut c)?;

                        let mut handlers = inner.handlers.lock().await;
                        let Some(handler) = handlers.get_mut(&id) else {
                            return Err(anyhow::Error::msg("received response for unknown id"));
                        };

                        handler
                            .send(bincode::Deserializer::with_reader(c, bin))
                            .await?;
                    }
                    Message::Ping(p) => ws.send(Message::Pong(p)).await?,
                    Message::Close(close) => {
                        if let Some(close) = close {
                            log::debug!("ws close, reason: {}", close.reason);
                        }
                        continue;
                    }
                    _ => continue,
                },
                Action::Send(msg) => {
                    ws.send(msg).await?;
                }
            }
        }
    }

    async fn incr_id(&self) -> u32 {
        let mut id_handle = self.id.lock().await;
        let id = *id_handle;
        *id_handle = id + 1;
        id
    }

    async fn send<C: cmd::Command>(&self, id: u32, cmd: C) -> Result<C::Return> {
        let cmd: cmd::Concrete = cmd.into();
        let data = bincode::Options::serialize(bincode::options(), &(id, cmd))?;
        self.sender.send(Message::Binary(data))?;

        if has_return::<C>() {
            let (tx, mut rx) = mpsc::channel(1);
            self.inner.handlers.lock().await.insert(id, tx);
            let mut de = rx.recv().await.unwrap();
            let re = C::Return::deserialize(&mut de)?;
            Ok(re)
        } else {
            unsafe { std::mem::zeroed() }
        }
    }
}

#[async_trait]
impl TransportAsync for Ws {
    async fn cmd<C: cmd::Command>(&self, cmd: C) -> Result<C::Return> {
        let id = self.incr_id().await;
        self.send(id, cmd).await
    }
}

#[async_trait]
impl SubscribableAsync for Ws {
    async fn subscribe<E: Event>(&self, ev: E) -> Result<broadcast::Receiver<E::Item>> {
        let id = self.incr_id().await;
        let ev: ConcreteType = ev.into();

        let (tx, mut worker_rx) = mpsc::channel(1);
        self.inner.handlers.lock().await.insert(id, tx);
        self.inner.events.lock().await.insert(ev, id);
        self.send(id, cmd::Subscribe(ev)).await?;

        let (client_tx, client_rx) = broadcast::channel(128);
        tokio::spawn(async move {
            while let Some(mut de) = worker_rx.recv().await {
                let item = E::Item::deserialize(&mut de)?;
                if client_tx.send(item).is_err() {
                    log::error!("no receiver for active subscription");
                };
            }
            anyhow::Ok(())
        });

        Ok(client_rx)
    }

    async fn unsubscribe<E>(&self, ev: E) -> Result<()>
    where
        E: Event,
    {
        let ev = ev.into();

        let mut lock = self.inner.events.lock().await;
        match lock.entry(ev) {
            std::collections::hash_map::Entry::Occupied(v) => {
                let id = v.remove();
                self.send(id, cmd::Unsubscribe(ev)).await?;
                self.inner.handlers.lock().await.remove(&id);
            }
            std::collections::hash_map::Entry::Vacant(_) => anyhow::bail!("Subscription not found"),
        };
        Ok(())
    }
}

impl Drop for Ws {
    fn drop(&mut self) {
        let _ = self.sender.send(Message::Close(None));
        let _ = block_on(self.handle.take().unwrap());
    }
}
