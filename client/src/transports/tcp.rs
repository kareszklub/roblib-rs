use super::{Subscribable, Transport};
use anyhow::Result;
use roblib::{
    cmd::{self, has_return, Command},
    event,
};
use serde::Deserialize;
use std::{collections::HashMap, io::Write, sync::Arc};

type D<'a> = bincode::Deserializer<
    bincode::de::read::IoReader<&'a std::net::TcpStream>,
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
        loop {
            let running = inner.running.read().unwrap();
            if !*running {
                return Ok(());
            }
            drop(running);

            let id: u32 = bincode::Options::deserialize_from(bin, &mut socket)?;
            let mut handlers = inner.handlers.lock().unwrap();

            let Some(handler) = handlers.get_mut(&id) else {
                return Err(anyhow::Error::msg("received response for unknown id"));
            };

            handler(bincode::Deserializer::with_reader(&socket, bin))?;
        }
    }

    fn cmd_id<C>(&self, cmd: C, id: u32) -> Result<C::Return>
    where
        C: Command,
        C::Return: Send + 'static,
    {
        let concrete: cmd::Concrete = cmd.into();
        let buf = bincode::Options::serialize(bincode::options(), &(id, concrete))?;
        (&self.socket).write_all(&(buf.len() as u32).to_be_bytes())?;
        (&self.socket).write_all(&buf)?;
        dbg!(&buf);

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
        C::Return: Send + 'static,
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
    fn subscribe<E, F>(&self, ev: E, mut handler: F) -> anyhow::Result<()>
    where
        E: roblib::event::Event,
        F: (FnMut(E::Item) -> anyhow::Result<()>) + Send + Sync + 'static,
    {
        let mut handlers = self.inner.handlers.lock().unwrap();
        let mut id_handle = self.id.lock().unwrap();

        let id = *id_handle;
        let ev = Into::<event::ConcreteType>::into(ev);
        let cmd: cmd::Concrete = cmd::Subscribe(ev).into();

        let already_contains = handlers
            .insert(
                id,
                Box::new(move |mut des| handler(E::Item::deserialize(&mut des)?)),
            )
            .is_some();

        if already_contains {
            return Err(anyhow::Error::msg("already subscribed to this event"));
        }

        bincode::Options::serialize_into(bincode::options(), &self.socket, &(id, cmd))?;

        *id_handle += 1;

        self.inner.events.lock().unwrap().insert(ev, id);

        Ok(())
    }

    fn unsubscribe<E: roblib::event::Event>(&self, ev: E) -> anyhow::Result<()> {
        let concrete_event = ev.into();
        let cmd: cmd::Concrete = cmd::Unsubscribe(concrete_event).into();

        bincode::serialize_into(&self.socket, &cmd)?;

        let mut evs = self.inner.events.lock().unwrap();
        let id = evs.remove(&concrete_event).unwrap();
        self.inner.handlers.lock().unwrap().remove(&id);

        Ok(())
    }
}
