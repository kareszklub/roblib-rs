use crate::Transport;
use anyhow::Result;
use roblib::{
    cmd::{self, has_return, Command},
    event::Event,
};
use serde::Deserialize;
use std::{
    collections::{hash_map::Entry, HashMap},
    io::Cursor,
    pin::Pin,
    sync::Arc,
};

use super::Subscribable;

type D<'a> = bincode::Deserializer<bincode::de::read::SliceReader<'a>, bincode::DefaultOptions>;
type H = Box<dyn for<'a> FnMut(D<'a>) -> Result<()> + Send + Sync>;

struct UdpSender {
    sock: std::net::UdpSocket,
    id: u32,
}

struct UdpInner {
    send: std::sync::Mutex<UdpSender>,
    handlers: std::sync::Mutex<HashMap<u32, H>>,
    running: std::sync::RwLock<bool>,
}

pub struct Udp {
    inner: Arc<UdpInner>,
}

impl Udp {
    pub fn new(robot: impl std::net::ToSocketAddrs) -> Result<Self> {
        let sock = std::net::UdpSocket::bind("0.0.0.0")?;
        sock.connect(robot)?;

        let inner = Arc::new(UdpInner {
            send: UdpSender { sock, id: 0 }.into(),
            handlers: HashMap::new().into(),
            running: true.into(),
        });

        let i2 = inner.clone();
        std::thread::spawn(|| Self::recieve(i2));

        Ok(Self { inner })
    }

    fn recieve(inner: Arc<UdpInner>) -> Result<()> {
        let mut buf = [0; 512];
        loop {
            let running = inner.running.read().unwrap();
            if !*running {
                break;
            }
            drop(running);

            let sender = inner.send.lock().unwrap();

            let len = sender.sock.recv(&mut buf)?;
            let buf = &buf[..len];

            let mut curs = Cursor::new(buf);
            let id: u32 = bincode::deserialize_from(&mut curs)?;
            if let Some(h) = inner.handlers.lock().unwrap().get_mut(&id) {
                let pos = curs.position() as usize;
                let rest = &curs.into_inner()[pos..];
                h(bincode::Deserializer::from_slice(rest, bincode::options()))?;
            }
        }
        Ok(())
    }
}

impl Transport for Udp {
    fn cmd<C: Command>(&self, cmd: C) -> Result<C::Return>
    where
        C::Return: Send,
    {
        let mut send = self.inner.send.lock().unwrap();
        let id = send.id;
        send.id += 1;

        let concrete: cmd::Concrete = cmd.into();
        send.sock.send(&bincode::serialize(&(id, concrete))?)?;
        drop(send);

        Ok(if has_return::<C>() {
            let (tx, rx) = std::sync::mpsc::sync_channel(1);

            let a: H = Box::new(move |mut des: D| {
                let r = C::Return::deserialize(&mut des)?;
                tx.send(r).unwrap();
                Ok::<(), anyhow::Error>(())
            });

            self.inner.handlers.lock().unwrap().insert(id, a);

            let ret = rx.recv()?;

            self.inner.handlers.lock().unwrap().remove(&id);

            ret
        } else {
            unsafe { std::mem::zeroed() }
        })
    }
}

impl Subscribable for Udp {
    fn subscribe<E, F>(&self, ev: E, mut handler: F) -> Result<()>
    where
        E: Event,
        F: FnMut(E::Item) -> Result<()>,
        F: Send + Sync + 'static,
    {
        let mut send = self.inner.send.lock().unwrap();
        let id = send.id;
        send.id += 1;

        let cmd: cmd::Concrete = cmd::Subscribe(ev.into()).into();

        send.sock.send(&bincode::serialize(&cmd)?)?;

        self.inner.handlers.lock().unwrap().insert(
            id,
            Box::new(move |mut des| handler(E::Item::deserialize(&mut des)?)),
        );

        Ok(())
    }

    fn unsubscribe<E: roblib::event::Event>(&self, ev: E) -> Result<()> {
        let mut send = self.inner.send.lock().unwrap();
        let id = send.id;
        send.id += 1;

        let cmd: cmd::Concrete = cmd::Unsubscribe(ev.into()).into();

        send.sock.send(&bincode::serialize(&cmd)?)?;
        drop(send);

        self.inner.handlers.lock().unwrap().remove(&id);

        Ok(())
    }
}

type HA = Box<
    dyn for<'a> FnMut(
            D<'a>,
        )
            -> Pin<Box<dyn std::future::Future<Output = Result<()>> + 'a + Send + Sync>>
        + Send
        + Sync,
>;
type HA1 = Box<
    dyn for<'a> FnOnce(
            D<'a>,
        )
            -> Pin<Box<dyn std::future::Future<Output = Result<()>> + 'a + Send + Sync>>
        + Send
        + Sync,
>;

#[cfg(feature = "async")]
struct UdpSenderAsync {
    sock: tokio::net::UdpSocket,
    id: u32,
}

enum EventHandler {
    Multi(HA),
    Single(HA1),
}

#[cfg(feature = "async")]
struct UdpInnerAsync {
    send: tokio::sync::Mutex<UdpSenderAsync>,
    handlers: tokio::sync::Mutex<HashMap<u32, EventHandler>>,
    running: tokio::sync::RwLock<bool>,
}

#[cfg(feature = "async")]
pub struct UdpAsync {
    inner: Arc<UdpInnerAsync>,
}

#[cfg(feature = "async")]
impl UdpAsync {
    pub async fn new(robot: impl tokio::net::ToSocketAddrs) -> Result<Self> {
        let sock = tokio::net::UdpSocket::bind("0.0.0.0").await?;
        sock.connect(robot).await?;

        let inner = Arc::new(UdpInnerAsync {
            send: UdpSenderAsync { sock, id: 0 }.into(),
            handlers: HashMap::new().into(),
            running: true.into(),
        });

        let i2 = inner.clone();
        tokio::spawn(Self::recieve(i2));

        Ok(Self { inner })
    }

    async fn recieve(inner: Arc<UdpInnerAsync>) -> Result<()> {
        let mut buf = [0; 512];
        loop {
            let running = inner.running.read().await;
            if !*running {
                break;
            }
            drop(running);

            let sender = inner.send.lock().await;

            let len = sender.sock.recv(&mut buf).await?;
            let buf = &buf[..len];

            let mut curs = Cursor::new(buf);
            let id: u32 = bincode::deserialize_from(&mut curs)?;

            let handlers = &mut inner.handlers.lock().await;

            if let Entry::Occupied(mut e) = handlers.entry(id) {
                let pos = curs.position() as usize;
                let rest = &curs.into_inner()[pos..];

                let des = bincode::Deserializer::from_slice(rest, bincode::options());

                if let EventHandler::Multi(m) = e.get_mut() {
                    m(des).await?;
                } else {
                    match e.remove() {
                        EventHandler::Single(s) => {
                            s(des).await?;
                        }
                        EventHandler::Multi(_) => unreachable!(),
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(feature = "async")]
#[cfg_attr(feature = "async", async_trait::async_trait)]
impl super::TransportAsync for UdpAsync {
    async fn cmd<C>(&self, cmd: C) -> Result<C::Return>
    where
        C: Command + Send + Sync,
        C::Return: Send + Sync,
    {
        let mut send = self.inner.send.lock().await;
        let id = send.id;
        send.id += 1;

        let cmd: cmd::Concrete = cmd.into();
        send.sock.send(&bincode::serialize(&cmd)?).await?;
        drop(send);

        Ok(if has_return::<C>() {
            let (tx, mut rx) = tokio::sync::mpsc::channel::<Result<<C as Command>::Return>>(1);

            let a: HA1 = Box::new(move |mut des: D| {
                let r = C::Return::deserialize(&mut des).map_err(anyhow::Error::new);

                Box::pin(async move { tx.send(r).await.map_err(anyhow::Error::new) })
            });

            self.inner
                .handlers
                .lock()
                .await
                .insert(id, EventHandler::Single(a));

            rx.recv().await.unwrap()?
        } else {
            unsafe { std::mem::zeroed() }
        })
    }
}

#[cfg(feature = "async")]
#[cfg_attr(feature = "async", async_trait::async_trait)]
impl super::SubscribableAsync for UdpAsync {
    async fn subscribe<E, F, R>(&self, ev: E, mut handler: F) -> Result<()>
    where
        E: Event + Send,
        E::Item: Send + Sync,
        F: FnMut(E::Item) -> R,
        F: Send + Sync + 'static,
        R: std::future::Future<Output = Result<()>> + Send + Sync,
    {
        let mut send = self.inner.send.lock().await;
        let id = send.id;
        send.id += 1;

        let cmd: cmd::Concrete = cmd::Subscribe(ev.into()).into();

        send.sock.send(&bincode::serialize(&cmd)?).await?;

        let ha: HA = Box::new(move |mut des| {
            Box::pin(async {
                let it = E::Item::deserialize(&mut des);
                match it {
                    Ok(i) => handler(i).await,
                    Err(e) => Err(anyhow::Error::new(e)),
                }
            })
        });

        self.inner
            .handlers
            .lock()
            .await
            .insert(id, EventHandler::Multi(ha));

        Ok(())
    }

    async fn unsubscribe<E: Event + Send>(&self, ev: E) -> Result<()> {
        let mut send = self.inner.send.lock().await;
        let id = send.id;
        send.id += 1;

        let cmd: cmd::Concrete = cmd::Unsubscribe(ev.into()).into();

        send.sock.send(&bincode::serialize(&cmd)?).await?;
        drop(send);

        self.inner.handlers.lock().await.remove(&id);

        Ok(())
    }
}
