use std::{io::Cursor, net::SocketAddr, sync::Arc, time::Duration};

use crate::{cmd::execute_concrete, Backends};
use roblib::{cmd, event::ConcreteValue};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, Interest},
    net::{TcpListener, TcpStream, ToSocketAddrs},
    spawn,
    sync::broadcast::{Receiver, Sender},
    task::JoinHandle,
};

pub type Id = SocketAddr;
pub type SubId = u32;
pub type Item = (Id, SubId);
pub type Tx = Sender<(ConcreteValue, Item)>;
pub type Rx = Receiver<(ConcreteValue, Item)>;

type Ret = Vec<JoinHandle<anyhow::Result<()>>>;

pub(crate) async fn start(
    addr: impl ToSocketAddrs,
    robot: Arc<Backends>,
    rx: Rx,
) -> anyhow::Result<JoinHandle<Ret>> {
    let server = TcpListener::bind(addr).await?;
    Ok(spawn(run(server, robot, rx)))
}

async fn run(server: TcpListener, robot: Arc<Backends>, rx: Rx) -> Ret {
    let mut handles = Vec::new();
    loop {
        let conn = tokio::select! {
            biased;
            _ = robot.abort_token.cancelled() => return handles,
            Ok(conn) = server.accept() => conn,
        };
        let h = spawn(handle_client(robot.clone(), conn, rx.resubscribe()));
        handles.push(h);
    }
}

enum Thing {
    ClientMessage(usize),
    Event(ConcreteValue, Item),
    Disconnect,
    ServerAbort,
}

async fn handle_client(
    robot: Arc<Backends>,
    (mut stream, addr): (TcpStream, SocketAddr),
    mut rx: Rx,
) -> anyhow::Result<()> {
    let bin = bincode::options();
    const HEADER: usize = std::mem::size_of::<u32>();

    let mut buf = vec![0; 512];
    let mut len = 0; // no. of bytes read for the current command we're attempting to parse
    let mut maybe_cmd_len = None;

    loop {
        let thing = tokio::select! {
            _ = robot.abort_token.cancelled() => Thing::ServerAbort,
            Ok(n) = stream.read(&mut buf[len..( HEADER + maybe_cmd_len.unwrap_or(0) )]) => Thing::ClientMessage(n),
            Ok(msg) = rx.recv() => Thing::Event(msg.0, msg.1),
            _ = tokio::time::sleep(Duration::from_secs(5)) => {
                let r = stream.ready(Interest::READABLE | Interest::WRITABLE).await;
                if r.map_or(true, |r| r.is_read_closed() || r.is_write_closed()) {
                    Thing::Disconnect
                } else { continue; }
            }
        };

        match thing {
            Thing::ClientMessage(n) => {
                if n == 0 {
                    log::debug!("received 0 sized msg, investigating disconnect");
                    // give the socket some time to fully realize disconnect
                    tokio::time::sleep(Duration::from_millis(50)).await;
                    let r = stream.ready(Interest::READABLE | Interest::WRITABLE).await;
                    if r.map_or(true, |r| r.is_read_closed() || r.is_write_closed()) {
                        log::debug!("tcp client disconnected: {addr}");
                        return Ok(());
                    }
                }

                len += n;
                log::debug!(
                    "Thing::ClientMessage - n: {n}, len: {len}, mblen: {maybe_cmd_len:?}, buflen: {}",
                    buf.len()
                );
                // not enough bytes to get command length
                if len < HEADER {
                    log::debug!("read more header");
                    continue;
                }

                let cmd_len = match maybe_cmd_len {
                    Some(n) => n,
                    None => {
                        let cmd = u32::from_be_bytes((&buf[..HEADER]).try_into().unwrap()) as usize;
                        // buf.resize(HEADER + cmd, 0);
                        maybe_cmd_len = Some(cmd);
                        log::debug!("header received, cmdlen: {cmd}");
                        cmd
                    }
                };

                // not enough bytes to parse command, get some more
                if len < HEADER + cmd_len {
                    log::debug!("read more command");
                    continue;
                }

                let (id, cmd): (u32, cmd::Concrete) =
                    bincode::Options::deserialize(bin, &buf[HEADER..len])?;

                let mut c = Cursor::new(&mut buf[..]);
                bincode::Options::serialize_into(bin, &mut c, &id)?;

                let res = execute_concrete(
                    cmd,
                    robot.clone(),
                    &mut bincode::Serializer::new(&mut c, bin),
                )
                .await?;

                if res.is_some() {
                    stream.write_all(&buf).await?;
                }

                // reset
                len = 0;
                maybe_cmd_len = None;
            }

            Thing::Event(ev, (ev_addr, id)) => {
                if addr != ev_addr {
                    continue;
                }
                let data = match ev {
                    #[cfg(feature = "roland")]
                    ConcreteValue::TrackSensor(v) => bincode::Options::serialize(bin, &(id, v)),
                    #[cfg(feature = "roland")]
                    ConcreteValue::UltraSensor(v) => bincode::Options::serialize(bin, &(id, v)),
                    #[cfg(feature = "gpio")]
                    ConcreteValue::GpioPin(v) => bincode::Options::serialize(bin, &(id, v)),
                    #[cfg(feature = "camloc")]
                    ConcreteValue::CamlocConnect(v) => bincode::Options::serialize(bin, &(id, v)),
                    #[cfg(feature = "camloc")]
                    ConcreteValue::CamlocDisconnect(v) => {
                        bincode::Options::serialize(bin, &(id, v))
                    }
                    #[cfg(feature = "camloc")]
                    ConcreteValue::CamlocPosition(v) => bincode::Options::serialize(bin, &(id, v)),
                    #[cfg(feature = "camloc")]
                    ConcreteValue::CamlocInfoUpdate(v) => {
                        bincode::Options::serialize(bin, &(id, v))
                    }
                    ConcreteValue::None => continue,
                }?;
                stream.write_all(&(data.len() as u32).to_be_bytes()).await?;
                stream.write_all(&data).await?;
            }

            Thing::Disconnect => {
                log::debug!("tcp client disconnected: {addr}");
                return Ok(());
            }
            Thing::ServerAbort => {
                log::debug!("abort: tcp {addr}");
                return Ok(());
            }
        }
    }
}
