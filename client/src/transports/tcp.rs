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
    handlers: std::sync::Mutex<HashMap<u32, (Handler, bool)>>,
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

            let Some(mut handler) = inner.handlers.lock().unwrap().remove(&id) else {
                return Err(anyhow::Error::msg("received response for unknown id"));
            };

            handler.0(bincode::Deserializer::with_reader(&mut c, bin))?;

            if handler.1 {
                inner.handlers.lock().unwrap().insert(id, handler);
            }
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
            self.inner.handlers.lock().unwrap().insert(id, (a, false));

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
        self.cmd_id(cmd, id)
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
            (
                Box::new(move |mut des| handler(E::Item::deserialize(&mut des)?)),
                true,
            ),
        );
        self.inner.events.lock().unwrap().insert(ev.clone(), id);

        self.cmd_id(cmd::Subscribe(ev), id)?;

        Ok(())
    }

    fn unsubscribe<E: roblib::event::Event>(&self, ev: E) -> Result<()> {
        let ev = ev.into();
        let cmd = cmd::Unsubscribe(ev.clone());

        let mut lock = self.inner.events.lock().unwrap();
        match lock.entry(ev) {
            std::collections::hash_map::Entry::Occupied(v) => {
                let id = v.remove();
                self.cmd_id(cmd, id)?;
                self.inner.handlers.lock().unwrap().remove(&id);
            }
            std::collections::hash_map::Entry::Vacant(_) => anyhow::bail!("Subscription not found"),
        }

        Ok(())
    }
}

#[cfg(feature = "async")]
pub use tcp_async::*;
#[cfg(feature = "async")]
pub mod tcp_async {
    use std::{collections::HashMap, io::Cursor, time::Duration};

    use crate::transports::{SubscribableAsync, TransportAsync};
    use anyhow::Result;
    use async_trait::async_trait;
    use roblib::{
        cmd::{self, has_return, Command},
        event::{self, Event},
    };
    use serde::{Deserialize, Serialize};
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt, Interest},
        net::{TcpStream, ToSocketAddrs},
        sync::{broadcast, mpsc, oneshot},
        task::JoinHandle,
    };

    type D = bincode::Deserializer<
        bincode::de::read::IoReader<Cursor<Vec<u8>>>,
        bincode::DefaultOptions,
    >;

    enum Action {
        ServerMessage(usize),
        Cmd(cmd::Concrete, Option<oneshot::Sender<D>>),
        Sub(event::ConcreteType, Option<mpsc::UnboundedSender<D>>),
    }

    struct Worker {
        stream: TcpStream,
        cmd_rx: mpsc::UnboundedReceiver<(cmd::Concrete, Option<oneshot::Sender<D>>)>,
        sub_rx: mpsc::UnboundedReceiver<(event::ConcreteType, Option<mpsc::UnboundedSender<D>>)>,
    }
    impl Worker {
        pub fn new(
            stream: TcpStream,
        ) -> (
            Self,
            mpsc::UnboundedSender<(cmd::Concrete, Option<oneshot::Sender<D>>)>,
            mpsc::UnboundedSender<(event::ConcreteType, Option<mpsc::UnboundedSender<D>>)>,
        ) {
            let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
            let (sub_tx, sub_rx) = mpsc::unbounded_channel();
            let s = Self {
                stream,
                cmd_rx,
                sub_rx,
            };
            (s, cmd_tx, sub_tx)
        }
        pub async fn worker(mut self) -> Result<()> {
            const HEADER: usize = std::mem::size_of::<u32>();

            let mut next_id = super::super::ID_START;
            let bin = bincode::options();
            let mut buf = vec![0; 512];
            let mut len = 0; // no. of bytes read for the current command we're attempting to parse
            let mut maybe_cmd_len = None;
            let mut cmds: HashMap<u32, oneshot::Sender<D>> = HashMap::new();
            let mut subs: HashMap<u32, mpsc::UnboundedSender<D>> = HashMap::new();
            let mut sub_ids: HashMap<event::ConcreteType, u32> = HashMap::new();
            loop {
                let action = tokio::select! {
                    Ok(n) = self.stream.read(&mut buf[len..( HEADER + maybe_cmd_len.unwrap_or(0) )]) => Action::ServerMessage(n),
                    Some(cmd) = self.cmd_rx.recv() => Action::Cmd(cmd.0, cmd.1),
                    Some(sub) = self.sub_rx.recv() => Action::Sub(sub.0, sub.1),
                    // _ = tokio::time::sleep(Duration::from_secs(5)) => {
                    //     self.check_disconnect().await;
                    //     continue;
                    // }
                };

                match action {
                    // adapted from server/src/transports/tcp.rs
                    Action::ServerMessage(n) => {
                        if n == 0 {
                            log::debug!("tcp: received 0 sized msg, investigating disconnect");
                            // give the socket some time to fully realize disconnect
                            tokio::time::sleep(Duration::from_millis(100)).await;
                            if self.check_disconnect().await {
                                anyhow::bail!("Server disconnected!");
                            }
                        }

                        len += n;
                        if len < HEADER {
                            continue;
                        }
                        let cmd_len = match maybe_cmd_len {
                            Some(n) => n,
                            None => {
                                let cmd = u32::from_be_bytes((&buf[..HEADER]).try_into().unwrap())
                                    as usize;
                                // buf.resize(HEADER + cmd, 0);
                                maybe_cmd_len = Some(cmd);
                                // log::debug!("header received, cmdlen: {cmd}");
                                cmd
                            }
                        };
                        if len < HEADER + cmd_len {
                            continue;
                        }

                        let mut c = Cursor::new(buf[HEADER..len].to_vec()); // clone :(
                        let id: u32 = bincode::Options::deserialize_from(bin, &mut c)?;
                        dbg!(id);
                        if let Some(tx) = subs.get(&id) {
                            dbg!("sub");
                            tx.send(bincode::Deserializer::with_reader(c, bin))?;
                        } else if let Some(tx) = cmds.remove(&id) {
                            dbg!("cmd");
                            if tx.send(bincode::Deserializer::with_reader(c, bin)).is_err() {
                                log::error!("cmd receiver dropped");
                            }
                        } else {
                            log::error!("server sent invalid id");
                        }

                        len = 0;
                        maybe_cmd_len = None;
                    }
                    Action::Cmd(cmd, maybe_tx) => {
                        let id = next_id;
                        next_id += 1;
                        if let Some(tx) = maybe_tx {
                            cmds.insert(id, tx);
                        }
                        self.send((id, cmd)).await?;
                    }
                    Action::Sub(ev, Some(tx)) => {
                        dbg!(&ev);
                        let id = next_id;
                        next_id += 1;
                        subs.insert(id, tx);
                        let cmd: cmd::Concrete = cmd::Subscribe(ev).into();
                        dbg!(&cmd);
                        self.send((id, cmd)).await?;
                        dbg!();
                    }
                    // None: unsubscribe
                    Action::Sub(ev, None) => {
                        let Some(id) = sub_ids.remove(&ev) else {
                            log::error!("unsubscribe failed: {ev:?} subscription not found");
                            continue;
                        };
                        subs.remove(&id);
                        let cmd: cmd::Concrete = cmd::Unsubscribe(ev).into();
                        self.send((id, cmd)).await?;
                    }
                }
            }
        }
        async fn check_disconnect(&mut self) -> bool {
            let r = self
                .stream
                .ready(Interest::READABLE | Interest::WRITABLE)
                .await;
            if r.map_or(true, |r| r.is_read_closed() || r.is_write_closed()) {
                log::error!("Server disconnected!");
                // TODO: handle failure
                return true;
            }
            return false;
        }
        async fn send(&mut self, data: impl Serialize) -> Result<()> {
            let buf = bincode::Options::serialize(bincode::options(), &data)?;
            self.stream
                .write_all(&(buf.len() as u32).to_be_bytes())
                .await?;
            self.stream.write_all(&buf).await?;
            Ok(())
        }
    }

    pub struct TcpAsync {
        _handle: Option<JoinHandle<Result<()>>>,
        cmd_tx: mpsc::UnboundedSender<(cmd::Concrete, Option<oneshot::Sender<D>>)>,
        sub_tx: mpsc::UnboundedSender<(event::ConcreteType, Option<mpsc::UnboundedSender<D>>)>,
    }

    impl TcpAsync {
        pub async fn connect(addr: impl ToSocketAddrs) -> Result<Self> {
            let stream = TcpStream::connect(addr).await?;
            let (worker, cmd_tx, sub_tx) = Worker::new(stream);
            let handle = Some(tokio::spawn(async {
                let r = worker.worker().await;
                eprintln!("worker dropped??");
                r
            }));

            Ok(Self {
                _handle: handle,
                cmd_tx,
                sub_tx,
            })
        }
    }

    #[async_trait]
    impl TransportAsync for TcpAsync {
        async fn cmd<C>(&self, cmd: C) -> Result<C::Return>
        where
            C: Command,
        {
            let concr: cmd::Concrete = cmd.into();
            if has_return::<C>() {
                let (tx, rx) = oneshot::channel();
                self.cmd_tx.send((concr, Some(tx)))?;
                let mut de = rx.await?;
                Ok(C::Return::deserialize(&mut de)?)
            } else {
                self.cmd_tx.send((concr, None))?;
                unsafe { std::mem::zeroed() }
            }
        }
    }

    #[async_trait]
    impl SubscribableAsync for TcpAsync {
        async fn subscribe<E: Event>(&self, ev: E) -> Result<broadcast::Receiver<E::Item>> {
            let (worker_tx, mut worker_rx) = mpsc::unbounded_channel();
            self.sub_tx.send((ev.into(), Some(worker_tx)))?;

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
            dbg!();
            Ok(client_rx)
        }

        async fn unsubscribe<E>(&self, ev: E) -> Result<()>
        where
            E: Event,
        {
            Ok(self.sub_tx.send((ev.into(), None))?)
        }
    }
}
