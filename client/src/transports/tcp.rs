use super::{Subscribable, Transport};
use anyhow::Result;
use roblib::{
    cmd::{self, has_return, Command},
    event::Event,
};
use serde::Deserialize;
use std::{
    collections::HashMap,
    io::{Cursor, Read, Write},
    sync::Arc,
};

type D<'a> = bincode::Deserializer<
    bincode::de::read::IoReader<&'a mut Cursor<&'a [u8]>>,
    bincode::DefaultOptions,
>;
type Handler = Box<dyn Send + Sync + (for<'a> FnMut(D<'a>) -> Result<()>)>;

struct TcpInner {
    handlers: std::sync::Mutex<HashMap<u32, Handler>>,
    events: std::sync::Mutex<HashMap<roblib::event::ConcreteType, u32>>,
    running: std::sync::RwLock<bool>,
}
pub struct Tcp {
    inner: Arc<TcpInner>,

    socket: std::net::TcpStream,
    id: std::sync::Mutex<u32>,
}

impl Tcp {
    const HEADER: usize = std::mem::size_of::<u32>();

    pub fn connect(robot: impl std::net::ToSocketAddrs) -> anyhow::Result<Self> {
        let socket = std::net::TcpStream::connect(robot)?;

        let inner = Arc::new(TcpInner {
            handlers: HashMap::new().into(),
            events: HashMap::new().into(),
            running: true.into(),
        });

        let inner_clone = inner.clone();
        let socket_clone = socket.try_clone()?;
        std::thread::spawn(|| Self::listen(inner_clone, socket_clone));

        Ok(Self {
            inner,
            id: super::ID_START.into(),
            socket,
        })
    }

    fn listen(inner: Arc<TcpInner>, mut socket: std::net::TcpStream) -> Result<()> {
        let bin = bincode::options();
        let mut buf = vec![0; 512];
        loop {
            let running = inner.running.read().unwrap();
            if !*running {
                return Ok(());
            }
            drop(running);

            socket.read_exact(&mut buf[..Self::HEADER])?;
            let len = u32::from_be_bytes(buf[..Self::HEADER].try_into()?) as usize;
            let end = Self::HEADER + len;
            // log::debug!("Receiving {len} bytes");
            if len > buf.len() {
                buf.resize(len, 0);
                log::debug!("Connection buffer resized to {len}");
            }
            socket.read_exact(&mut buf[Self::HEADER..end])?;

            let mut c = Cursor::new(&buf[Self::HEADER..end]);
            let id: u32 = bincode::Options::deserialize_from(bin, &mut c)?;

            let mut handlers = inner.handlers.lock().unwrap();
            let Some(handler) = handlers.get_mut(&id) else {
                return Err(anyhow::Error::msg("received response for unknown id"));
            };

            handler(bincode::Deserializer::with_reader(&mut c, bin))?;
        }
    }

    fn cmd_id<C>(&self, cmd: C, id: u32) -> Result<C::Return>
    where
        C: Command,
    {
        let concrete: cmd::Concrete = cmd.into();
        let buf = bincode::Options::serialize(bincode::options(), &(id, concrete))?;
        (&self.socket).write_all(&(buf.len() as u32).to_be_bytes())?;
        (&self.socket).write_all(&buf)?;

        Ok(if has_return::<C>() {
            let (tx, rx) = std::sync::mpsc::sync_channel(1);

            let a: Handler = Box::new(move |mut des: D| {
                let r = C::Return::deserialize(&mut des)?;
                tx.send(r).unwrap();
                Ok::<(), anyhow::Error>(())
            });

            self.inner.handlers.lock().unwrap().insert(id, a);

            rx.recv()?
        } else {
            unsafe { std::mem::zeroed() }
        })
    }
}

impl Transport for Tcp {
    fn cmd<C>(&self, cmd: C) -> anyhow::Result<C::Return>
    where
        C: Command,
    {
        let mut id_handle = self.id.lock().unwrap();
        let id = *id_handle;
        *id_handle = id + 1;
        drop(id_handle);

        let res = self.cmd_id(cmd, id);
        self.inner.handlers.lock().unwrap().remove(&id);
        res
    }
}

impl Subscribable for Tcp {
    fn subscribe<E, F>(&self, ev: E, mut handler: F) -> Result<()>
    where
        E: Event,
        F: (FnMut(E::Item) -> Result<()>) + Send + Sync + 'static,
    {
        let mut id_handle = self.id.lock().unwrap();
        let id = *id_handle;
        *id_handle = id + 1;
        drop(id_handle);

        let ev = ev.into();

        self.inner.handlers.lock().unwrap().insert(
            id,
            Box::new(move |mut des| handler(E::Item::deserialize(&mut des)?)),
        );
        self.inner.events.lock().unwrap().insert(ev, id);

        self.cmd_id(cmd::Subscribe(ev), id)?;

        Ok(())
    }

    fn unsubscribe<E: roblib::event::Event>(&self, ev: E) -> Result<()> {
        let ev = ev.into();
        let cmd = cmd::Unsubscribe(ev);

        let mut lock = self.inner.events.lock().unwrap();
        match lock.entry(ev) {
            std::collections::hash_map::Entry::Occupied(v) => {
                let id = v.remove();
                dbg!((id, &cmd));
                self.cmd_id(cmd, id)?;
                self.inner.handlers.lock().unwrap().remove(&id);
            }
            std::collections::hash_map::Entry::Vacant(_) => anyhow::bail!("Subscription not found"),
        }

        Ok(())
    }
}
