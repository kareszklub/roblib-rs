use crate::Transport;
use anyhow::Result;
use roblib::{
    cmd::{self, has_return, Command},
    event::{self, Event},
};
use serde::Deserialize;
use std::{collections::HashMap, io::Cursor, sync::Arc};

use super::Subscribable;

type D<'a> = bincode::Deserializer<bincode::de::read::SliceReader<'a>, bincode::DefaultOptions>;
type Handler = Box<dyn (for<'a> FnMut(D<'a>) -> Result<()>) + Send + Sync>;

struct UdpInner {
    events: std::sync::Mutex<HashMap<roblib::event::ConcreteType, u32>>,
    handlers: std::sync::Mutex<HashMap<u32, Handler>>,
    running: std::sync::RwLock<bool>,
}

pub struct Udp {
    inner: Arc<UdpInner>,

    sock: std::net::UdpSocket,
    id: std::sync::Mutex<u32>,
}

impl Udp {
    pub fn connect(addr: impl std::net::ToSocketAddrs) -> Result<Self> {
        let sock = std::net::UdpSocket::bind("0.0.0.0:0")?;
        sock.connect(addr)?;

        let sock2 = sock.try_clone()?;

        let inner = Arc::new(UdpInner {
            handlers: HashMap::new().into(),
            events: HashMap::new().into(),
            running: true.into(),
        });

        let i2 = inner.clone();
        std::thread::spawn(move || Self::recieve(i2, sock2));

        Ok(Self {
            id: 0.into(),
            inner,
            sock,
        })
    }

    fn recieve(inner: Arc<UdpInner>, sock: std::net::UdpSocket) -> Result<()> {
        let mut buf = [0; 512];
        loop {
            let running = inner.running.read().unwrap();
            if !*running {
                break;
            }
            drop(running);

            let len = sock.recv(&mut buf)?;
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
    fn cmd<C>(&self, cmd: C) -> Result<C::Return>
    where
        C: Command,
        C::Return: Send + 'static,
    {
        let mut id_handle = self.id.lock().unwrap();
        let id = *id_handle;
        *id_handle = id + 1;

        let concrete: cmd::Concrete = cmd.into();
        self.sock.send(&bincode::serialize(&(id, concrete))?)?;

        Ok(if has_return::<C>() {
            let (tx, rx) = std::sync::mpsc::sync_channel(1);

            let a: Handler = Box::new(move |mut des: D| {
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
        F: (FnMut(E::Item) -> Result<()>) + Send + Sync + 'static,
    {
        let mut id_handle = self.id.lock().unwrap();
        let id = *id_handle;
        *id_handle += id + 1;

        let ev = Into::<event::ConcreteType>::into(ev);
        let cmd: cmd::Concrete = cmd::Subscribe(ev).into();

        self.sock.send(&bincode::serialize(&cmd)?)?;

        self.inner.handlers.lock().unwrap().insert(
            id,
            Box::new(move |mut des| handler(E::Item::deserialize(&mut des)?)),
        );

        Ok(())
    }

    fn unsubscribe<E: roblib::event::Event>(&self, ev: E) -> Result<()> {
        let ev = ev.into();
        let cmd: cmd::Concrete = cmd::Unsubscribe(ev.clone()).into();

        self.sock.send(&bincode::serialize(&cmd)?)?;

        let id = self.inner.events.lock().unwrap().remove(&ev).unwrap();

        self.inner.handlers.lock().unwrap().remove(&id);

        Ok(())
    }
}
