use crate::transports::{self, SubscriptionId};
use roblib::event::{self, ConcreteType, ConcreteValue};
use std::{collections::HashMap, sync::Arc};
use sub::SubStatus;
use tokio::sync::RwLock;

/// another channel to handle changes to subscriptions
/// sent by the transport layer, received by the event bus sender workers
#[allow(unused)]
pub mod sub {
    use crate::transports::SubscriptionId;
    use tokio::sync::broadcast::{Receiver, Sender};

    #[derive(Debug, Clone)]
    pub enum SubStatus {
        Subscribe,
        Unsubscribe,
        Disconnect,
    }
    /// (event, true for sub, false for unsub)
    pub type Item = (roblib::event::ConcreteType, SubscriptionId, SubStatus);
    pub type Tx = Sender<Item>;
    pub type Rx = Receiver<Item>;
}

pub(crate) struct EventBus {
    pub(self) robot: Arc<crate::Backends>,
    pub clients: RwLock<HashMap<ConcreteType, Vec<SubscriptionId>>>,

    pub bus_tcp: transports::tcp::Tx,
    pub bus_udp: transports::udp::Tx,
    pub bus_ws: transports::ws::Tx,
}
#[allow(dead_code)]
impl EventBus {
    pub fn new(
        robot: Arc<crate::Backends>,
        bus_tcp: transports::tcp::Tx,
        bus_udp: transports::udp::Tx,
        bus_ws: transports::ws::Tx,
    ) -> Self {
        Self {
            robot,
            clients: RwLock::new(HashMap::new()),
            bus_tcp,
            bus_udp,
            bus_ws,
        }
    }

    pub async fn resolve_send(&self, event: (ConcreteType, ConcreteValue)) {
        let clients = self.clients.read().await;
        let Some(v) = clients.get(&event.0) else {
            log::error!("NO CLIENT FOR {:?}", &event.0);
            return;
        };
        self.send_all(event, v)
    }

    pub fn resolve_send_blocking(&self, event: (ConcreteType, ConcreteValue)) {
        let clients = self.clients.blocking_read();
        let Some(v) = clients.get(&event.0) else {
            return;
        };
        self.send_all(event, v)
    }

    fn send(&self, event: (ConcreteType, ConcreteValue), client: &SubscriptionId) {
        match client {
            SubscriptionId::Tcp(addr, id) => {
                self.bus_tcp.send((event.1.clone(), (*addr, *id))).unwrap();
            }
            SubscriptionId::Udp(addr, id) => {
                self.bus_udp.send((event.1.clone(), (*addr, *id))).unwrap()
            }
            SubscriptionId::Ws(addr, id) => {
                self.bus_ws.send((event.1.clone(), (*addr, *id))).unwrap();
            }
        }
    }

    fn send_all(&self, event: (ConcreteType, ConcreteValue), clients: &Vec<SubscriptionId>) {
        for client in clients {
            self.send(event.clone(), client);
        }
    }
}

pub(crate) async fn init(
    robot: Arc<crate::Backends>,
    bus_tcp: transports::tcp::Tx,
    bus_udp: transports::udp::Tx,
    bus_ws: transports::ws::Tx,
) -> anyhow::Result<()> {
    let token = robot.abort_token.clone();
    let event_bus = Arc::new(EventBus::new(robot, bus_tcp, bus_udp, bus_ws));

    #[cfg(all(feature = "roland", feature = "backend"))]
    let h2 = if event_bus.robot.roland.is_some() {
        Some(tokio::spawn(connect_roland(event_bus.clone())))
    } else {
        None
    };

    let h1 = tokio::spawn(connect(event_bus));

    token.cancelled().await;
    log::debug!("abort: event_bus");

    h1.abort();

    #[cfg(all(feature = "roland", feature = "backend"))]
    if let Some(handle) = h2 {
        handle.abort();
    }

    Ok(())
}

/// hook up all the "inputs" (backends) to the event bus
pub(super) async fn connect(event_bus: Arc<EventBus>) {
    let mut subscribe = event_bus.robot.sub.subscribe();
    while let Ok((ty, id, sub)) = subscribe.recv().await {
        let mut clients = event_bus.clients.write().await;

        if let SubStatus::Disconnect = sub {
            for (ty, v) in clients.iter_mut() {
                v.retain(|s| !s.same_client(&id));
                if v.is_empty() {
                    cleanup_resource(&event_bus, ty.clone()).await;
                }
            }
            continue;
        }

        let ids = clients.entry(ty.clone()).or_default();

        match sub {
            SubStatus::Disconnect => unreachable!(),

            SubStatus::Subscribe => {
                dbg!(&ty);
                if ids.is_empty() {
                    create_resource(&event_bus, ty).await;
                } else if ids.contains(&id) {
                    log::error!("Attempted double subscription on event {ty:?}");
                    continue;
                }
                ids.push(id);
            }
            SubStatus::Unsubscribe => {
                if ids.is_empty() {
                    log::error!("Tried to unsubscribe from empty event");
                    continue;
                }

                let Some(i) = ids.iter().position(|x| x == &id) else {
                    log::error!("Tried to unsubscribe but was never subscribed");
                    continue;
                };

                ids.remove(i);

                if ids.is_empty() {
                    cleanup_resource(&event_bus, ty).await;
                }
            }
        }
    }

    log::error!("event_bus_sub dropped");
}

#[allow(unused_variables)]
async fn create_resource(event_bus: &Arc<EventBus>, ty: ConcreteType) {
    match ty {
        #[cfg(feature = "gpio")]
        ConcreteType::GpioPin(p) => {
            #[cfg(feature = "backend")]
            {
                struct Sub(Arc<EventBus>);
                impl roblib::gpio::backend::simple::Subscriber for Sub {
                    fn handle(&self, event: roblib::gpio::event::Event) {
                        let msg = match event {
                            roblib::gpio::event::Event::PinChanged(pin, value) => (
                                ConcreteType::GpioPin(roblib::gpio::event::GpioPin(pin)),
                                ConcreteValue::GpioPin(value),
                            ),
                        };

                        self.0.resolve_send_blocking(msg)
                    }
                }

                if let Some(r) = &event_bus.robot.raw_gpio {
                    if let Err(e) = r.subscribe(p.0, Box::new(Sub(event_bus.clone()))) {
                        log::error!("Failed to subscribe to gpio pin: {e}");
                    }
                }
            }
        }

        #[cfg(feature = "roland")]
        ConcreteType::TrackSensor(_) | ConcreteType::UltraSensor(_) => (),

        #[cfg(feature = "camloc")]
        a @ (ConcreteType::CamlocConnect(_)
        | ConcreteType::CamlocDisconnect(_)
        | ConcreteType::CamlocPosition(_)
        | ConcreteType::CamlocInfoUpdate(_)) => {
            #[cfg(feature = "backend")]
            {
                let Some(c) = &event_bus.robot.camloc else {
                    return;
                };

                let eb = event_bus.clone();
                let camloc_events = tokio::spawn(async move {
                    let c = eb.robot.camloc.as_ref().unwrap();
                    let mut chan = c.get_event_channel();

                    loop {
                        let rec = tokio::select! {
                            _ = eb.robot.abort_token.cancelled() => break,
                            rec = chan.recv() => rec,
                        };

                        let ev = match rec {
                            Ok(v) => v,
                            Err(e) => match e {
                                tokio::sync::broadcast::error::RecvError::Lagged(by) => {
                                    log::error!("Camloc events lagging by {by}");
                                    continue;
                                }
                                tokio::sync::broadcast::error::RecvError::Closed => {
                                    log::error!("Camloc events shutting down (channel was closed)");
                                    break;
                                }
                            },
                        };

                        use roblib::camloc::service::Event;
                        let ev = match ev {
                            Event::Connect(to, cam) => (
                                ConcreteType::CamlocConnect(event::CamlocConnect),
                                ConcreteValue::CamlocConnect((to, cam)),
                            ),
                            Event::Disconnect(from) => (
                                ConcreteType::CamlocDisconnect(event::CamlocDisconnect),
                                ConcreteValue::CamlocDisconnect(from),
                            ),
                            Event::PositionUpdate(pos) => (
                                ConcreteType::CamlocPosition(event::CamlocPosition),
                                ConcreteValue::CamlocPosition(pos),
                            ),
                            Event::InfoUpdate(who, info) => (
                                ConcreteType::CamlocInfoUpdate(event::CamlocInfoUpdate),
                                ConcreteValue::CamlocInfoUpdate((who, info)),
                            ),
                        };

                        eb.resolve_send(ev).await;
                    }
                });
            }
        }

        ConcreteType::None => unreachable!(),
    }
}

#[allow(unused_variables)]
async fn cleanup_resource(event_bus: &Arc<EventBus>, ty: ConcreteType) {
    match ty {
        #[cfg(feature = "roland")]
        ConcreteType::TrackSensor(_) | ConcreteType::UltraSensor(_) => (),

        #[cfg(feature = "gpio")]
        ConcreteType::GpioPin(p) =>
        {
            #[cfg(feature = "backend")]
            if let Some(r) = &event_bus.robot.raw_gpio {
                if let Err(e) = r.unsubscribe(p.0) {
                    log::error!("Failed to unsubscribe from gpio pin: {e}");
                }
            }
        }

        #[cfg(feature = "camloc")]
        ConcreteType::CamlocConnect(_)
        | ConcreteType::CamlocDisconnect(_)
        | ConcreteType::CamlocPosition(_)
        | ConcreteType::CamlocInfoUpdate(_) => (),

        ConcreteType::None => unreachable!(),
    }
}

#[cfg(all(feature = "roland", feature = "backend"))]
async fn connect_roland(event_bus: Arc<EventBus>) -> anyhow::Result<()> {
    use std::time::{Duration, Instant};

    use roblib::roland::Roland;
    use tokio::sync::broadcast::error::RecvError;
    use SubStatus::*;

    let mut rx = event_bus.robot.sub.subscribe();

    let roland = event_bus.robot.roland.as_ref().unwrap();

    let mut track_sensor_state = roland.track_sensor()?;

    roland.setup_tracksensor_interrupts()?;

    let mut track_subs = 0;

    struct UltraScheduleData {
        id: SubscriptionId,
        next: Instant,
        interval: Duration,
    }
    let mut ultra = vec![];

    loop {
        if track_subs + ultra.len() == 0 {
            let (ty, id, sub) = match rx.recv().await {
                Ok(v) => v,
                Err(RecvError::Closed) => return Err(anyhow::anyhow!("sub channel closed")),
                Err(RecvError::Lagged(n)) => {
                    error!("sub channel skipping {n}");
                    continue;
                }
            };

            if let SubStatus::Subscribe = sub {
                match ty {
                    ConcreteType::TrackSensor(_) => track_subs += 1,
                    ConcreteType::UltraSensor(event::UltraSensor(interval)) => {
                        ultra.push(UltraScheduleData {
                            id,
                            interval,
                            next: Instant::now() + interval,
                        });
                    }
                    _ => continue,
                }
            }
        }

        while let Ok((ty, id, sub)) = rx.try_recv() {
            match ty {
                ConcreteType::TrackSensor(_) => match sub {
                    Subscribe => track_subs += 1,
                    Unsubscribe => track_subs = track_subs.saturating_sub(1),
                    Disconnect => unreachable!(),
                },

                ConcreteType::UltraSensor(event::UltraSensor(interval)) => match sub {
                    Subscribe => {
                        ultra.push(UltraScheduleData {
                            next: Instant::now() + interval,
                            interval,
                            id,
                        });
                    }
                    Unsubscribe => {
                        ultra
                            .iter()
                            .position(|u| u.id == id)
                            .map(|p| ultra.remove(p));
                    }
                    Disconnect => unreachable!(),
                },

                _ => (),
            }
        }

        const MAX_WAIT: Duration = Duration::from_millis(200);

        let now = Instant::now();

        let next_ultra = ultra.iter_mut().min_by_key(|d| d.next);

        let poll_dur = if let Some(next_ultra) = &next_ultra {
            (next_ultra.next - now)
                .saturating_sub(roblib::roland::backend::constants::ultra_sensor::BLAST_DURATION)
        } else {
            MAX_WAIT
        };

        if track_subs > 0 {
            let poll_fn = {
                let robot = event_bus.robot.clone();
                move || {
                    robot
                        .roland
                        .as_ref()
                        .unwrap()
                        .poll_tracksensor(Some(poll_dur))
                }
            };
            let res = tokio::task::spawn_blocking(poll_fn).await??;

            if let Some((track_index, val)) = res {
                track_sensor_state[track_index] = val;

                event_bus
                    .resolve_send((
                        ConcreteType::TrackSensor(event::TrackSensor),
                        ConcreteValue::TrackSensor(track_sensor_state),
                    ))
                    .await;

                tokio::time::sleep_until((now + poll_dur).into()).await;
            }
        } else {
            tokio::time::sleep(poll_dur).await;
        }

        if let Some(next_ultra) = next_ultra {
            let ultra_fn = {
                let robot = event_bus.robot.clone();
                move || robot.roland.as_ref().unwrap().ultra_sensor()
            };

            let res = tokio::task::spawn_blocking(ultra_fn).await??;

            event_bus.send(
                (
                    ConcreteType::UltraSensor(event::UltraSensor(next_ultra.interval)),
                    ConcreteValue::UltraSensor(res),
                ),
                &next_ultra.id,
            );

            next_ultra.next += next_ultra.interval;
        }

        // TODO: batching
    }
}
