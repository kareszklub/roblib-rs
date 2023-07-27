use std::{
    collections::{hash_map::Entry, HashMap},
    io::Cursor,
    net::SocketAddr,
    sync::Arc,
};

use actix::spawn;
use actix_web::rt::net::UdpSocket;
use anyhow::Result;
use roblib::{cmd, event::ConcreteType};
use tokio::{net::ToSocketAddrs, sync::RwLock};

use crate::{cmd::execute_concrete, Backends};

pub(crate) async fn start(addr: impl ToSocketAddrs, robot: Arc<Backends>) -> Result<()> {
    let server = UdpSocket::bind(addr).await?;
    spawn(run(server, robot)).await??;

    Ok(())
}

type EventMap = HashMap<ConcreteType, Vec<(SocketAddr, u32)>>;

async fn run(server: UdpSocket, robot: Arc<Backends>) -> Result<()> {
    let mut buf = [0u8; 1024];

    let clients: Arc<RwLock<EventMap>> = Arc::new(HashMap::new().into());

    let server = Arc::new(server);
    spawn(handle_event(
        robot.event_bus.rx.resubscribe(),
        server.clone(),
        clients.clone(),
    ));

    loop {
        let (len, addr) = server.recv_from(&mut buf).await?;

        let (id, cmd): (u32, cmd::Concrete) = bincode::deserialize(&buf[..len])?;

        match cmd {
            cmd::Concrete::Subscribe(c) => {
                let mut clients = clients.write().await;

                match clients.entry(c.0.clone()) {
                    Entry::Occupied(mut o) => o.get_mut().push((addr, id)),
                    Entry::Vacant(v) => {
                        v.insert(vec![(addr, id)]);

                        robot
                            .event_bus
                            .sub_tx
                            .send((c.0, true))
                            .expect("event_bus_sub: no receivers");
                    }
                }

                continue;
            }
            cmd::Concrete::Unsubscribe(c) => {
                let mut clients = clients.write().await;

                match clients.entry(c.0) {
                    Entry::Occupied(mut o) => {
                        let o = o.get_mut();

                        if let Some(i) = o.iter().position(|x| x == (&(addr, id))) {
                            o.remove(i);
                        } else {
                            todo!()
                        }
                    }
                    Entry::Vacant(v) => {
                        v.insert(vec![(addr, id)]);
                    }
                }

                continue;
            }

            _ => (),
        }

        let mut c = Cursor::new(&mut buf[..]);
        bincode::serialize_into(&mut c, &id)?;

        let res = execute_concrete(
            cmd,
            robot.clone(),
            &mut bincode::Serializer::new(&mut c, bincode::DefaultOptions::new()),
        )
        .await?;

        if res.is_some() {
            server.send_to(&buf, addr).await?;
        }
    }
}

async fn handle_event(
    mut event_bus: crate::event_bus::Rx,
    event_send: Arc<UdpSocket>,
    clients: Arc<RwLock<EventMap>>,
) -> Result<()> {
    while let Ok((ty, ev)) = event_bus.recv().await {
        let clients = &clients.read().await;
        let Some(a) = clients.get(&ty) else {
            continue;
        };

        let val: Vec<u8> = match ev {
            #[cfg(feature = "gpio")]
            roblib::event::ConcreteValue::GpioPin(val) => bincode::serialize(&val)?,

            #[cfg(feature = "camloc")]
            roblib::event::ConcreteValue::CamlocConnect(val) => bincode::serialize(&val)?,
            #[cfg(feature = "camloc")]
            roblib::event::ConcreteValue::CamlocDisconnect(val) => bincode::serialize(&val)?,
            #[cfg(feature = "camloc")]
            roblib::event::ConcreteValue::CamlocPosition(val) => bincode::serialize(&val)?,
            #[cfg(feature = "camloc")]
            roblib::event::ConcreteValue::CamlocInfoUpdate(val) => bincode::serialize(&val)?,

            roblib::event::ConcreteValue::None => continue,
        };

        for (addr, id) in a {
            // TODO: cloning...
            let mut buf = bincode::serialize(id)?;
            buf.append(&mut val.clone());

            event_send.send_to(&buf, addr).await?;
        }
    }

    Ok(())
}
