use super::SubscriptionId;
use crate::{cmd::execute_concrete, event_bus::sub::SubStatus, Backends};
use actix_web::rt::net::UdpSocket;
use anyhow::Result;
use roblib::{cmd, event::ConcreteValue};
use std::{io::Cursor, net::SocketAddr, sync::Arc};
use tokio::{
    net::ToSocketAddrs,
    spawn,
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    task::JoinHandle,
};

pub type Id = SocketAddr;
pub type SubId = u32;
pub type Item = (Id, SubId);
pub type Tx = UnboundedSender<(ConcreteValue, Item)>;
pub type Rx = UnboundedReceiver<(ConcreteValue, Item)>;

pub(crate) async fn start(
    addr: impl ToSocketAddrs,
    robot: Arc<Backends>,
    rx: Rx,
) -> Result<(JoinHandle<Result<()>>, JoinHandle<Result<()>>)> {
    let socket = Arc::new(UdpSocket::bind(addr).await?);

    let server = spawn(run(socket.clone(), robot));
    let event_handler = spawn(handle_event(rx, socket));

    Ok((server, event_handler))
}

async fn run(server: Arc<UdpSocket>, robot: Arc<Backends>) -> Result<()> {
    let bin = bincode::options();
    let mut buf = [0u8; 1024];

    loop {
        let (len, addr) = server.recv_from(&mut buf).await?;

        let (id, cmd): (u32, cmd::Concrete) = bincode::Options::deserialize(bin, &buf[..len])?;

        match cmd {
            cmd::Concrete::Subscribe(c) => {
                let sub = SubscriptionId::Udp(addr, id);
                if let Err(e) = robot.sub.send((c.0, sub, SubStatus::Subscribe)) {
                    log::error!("event bus sub error: {e}");
                };
                continue;
            }
            cmd::Concrete::Unsubscribe(c) => {
                let sub = SubscriptionId::Udp(addr, id);
                if let Err(e) = robot.sub.send((c.0, sub, SubStatus::Unsubscribe)) {
                    log::error!("event bus sub error: {e}");
                };
                continue;
            }

            _ => (),
        }

        let mut c = Cursor::new(&mut buf[..]);
        bincode::Options::serialize_into(bin, &mut c, &id)?;

        let res = execute_concrete(
            cmd,
            robot.clone(),
            &mut bincode::Serializer::new(&mut c, bin),
        )
        .await?;

        if res.is_some() {
            server.send_to(&buf, addr).await?;
        }
    }
}

async fn handle_event(mut event_bus: Rx, event_send: Arc<UdpSocket>) -> Result<()> {
    let bin = bincode::options();
    while let Some((ev, (addr, id))) = event_bus.recv().await {
        let val: Vec<u8> = match ev {
            #[cfg(feature = "roland")]
            roblib::event::ConcreteValue::TrackSensor(val) => {
                bincode::Options::serialize(bin, &(id, val))?
            }
            #[cfg(feature = "roland")]
            roblib::event::ConcreteValue::UltraSensor(val) => {
                bincode::Options::serialize(bin, &(id, val))?
            }

            #[cfg(feature = "gpio")]
            roblib::event::ConcreteValue::GpioPin(val) => {
                bincode::Options::serialize(bin, &(id, val))?
            }

            #[cfg(feature = "camloc")]
            roblib::event::ConcreteValue::CamlocConnect(val) => {
                bincode::Options::serialize(bin, &(id, val))?
            }
            #[cfg(feature = "camloc")]
            roblib::event::ConcreteValue::CamlocDisconnect(val) => {
                bincode::Options::serialize(bin, &(id, val))?
            }
            #[cfg(feature = "camloc")]
            roblib::event::ConcreteValue::CamlocPosition(val) => {
                bincode::Options::serialize(bin, &(id, val))?
            }
            #[cfg(feature = "camloc")]
            roblib::event::ConcreteValue::CamlocInfoUpdate(val) => {
                bincode::Options::serialize(bin, &(id, val))?
            }

            roblib::event::ConcreteValue::None => continue,
        };

        event_send.send_to(&val, addr).await?;
    }
    Ok(())
}
