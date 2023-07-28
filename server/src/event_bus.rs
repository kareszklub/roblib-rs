use roblib::event::{ConcreteType, ConcreteValue};
use std::{collections::HashMap, sync::Arc};
use sub::SubStatus;
use tokio::sync::RwLock;

use crate::transports::{self, SubscriptionId};

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

    pub bus_udp: transports::udp::Tx,
}
impl EventBus {
    pub fn new(robot: Arc<crate::Backends>, bus_udp: transports::udp::Tx) -> Self {
        Self {
            robot,
            clients: RwLock::new(HashMap::new()),
            bus_udp,
        }
    }

    #[allow(unused)]
    pub async fn resolve_send(&self, event: (ConcreteType, ConcreteValue)) -> anyhow::Result<()> {
        let clients = self.clients.read().await;
        let Some(v) = clients.get(&event.0) else {
            return Ok(());
        };
        self.send_all(event, v)
    }

    pub fn resolve_send_blocking(
        &self,
        event: (ConcreteType, ConcreteValue),
    ) -> anyhow::Result<()> {
        let clients = self.clients.blocking_read();
        let Some(v) = clients.get(&event.0) else {
            return Ok(());
        };
        self.send_all(event, v)
    }

    fn send(
        &self,
        event: (ConcreteType, ConcreteValue),
        client: &SubscriptionId,
    ) -> anyhow::Result<()> {
        match client {
            SubscriptionId::Udp(addr, id) => self.bus_udp.send((event.1.clone(), (*addr, *id)))?,
        }
        Ok(())
    }

    fn send_all(
        &self,
        event: (ConcreteType, ConcreteValue),
        clients: &Vec<SubscriptionId>,
    ) -> anyhow::Result<()> {
        for client in clients {
            self.send(event.clone(), client)?;
        }
        Ok(())
    }
}

/// hook up all the "inputs" (backends) to the event bus
pub(super) async fn connect(event_bus: EventBus) {
    let event_bus = Arc::new(event_bus);

    #[cfg(all(feature = "roland", feature = "backend"))]
    if event_bus.robot.roland.is_some() {
        let event_bus = event_bus.clone();
        std::thread::spawn(move || connect_roland(event_bus));
    }

    // handle client subscription state changes
    // TODO: on client disconnect, unsubscribe them from all events
    while let Ok((ty, id, sub)) = event_bus.robot.sub.subscribe().recv().await {
        let mut clients = event_bus.clients.write().await;

        if let SubStatus::Disconnect = sub {
            for (ty, v) in clients.iter_mut() {
                v.retain(|s| !s.same_client(&id));
                if v.is_empty() {
                    cleanup_resource(&event_bus, *ty);
                }
            }
            continue;
        }

        let ids = clients.entry(ty).or_default();

        match sub {
            SubStatus::Disconnect => unreachable!(),

            SubStatus::Subscribe => {
                if ids.is_empty() {
                    create_resource(&event_bus, ty);
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
                    cleanup_resource(&event_bus, ty);
                }
            }
        }
    }

    log::error!("event_bus_sub dropped");
}

fn create_resource(event_bus: &Arc<EventBus>, ty: ConcreteType) {
    match ty {
        #[cfg(all(feature = "gpio", feature = "backend"))]
        ConcreteType::GpioPin(p) => {
            struct Sub(Arc<EventBus>);
            impl roblib::gpio::backend::simple::Subscriber for Sub {
                fn handle(&self, event: roblib::gpio::event::Event) {
                    let msg = match event {
                        roblib::gpio::event::Event::PinChanged(pin, value) => (
                            ConcreteType::GpioPin(roblib::gpio::event::GpioPin(pin)),
                            ConcreteValue::GpioPin(value),
                        ),
                    };

                    if let Err(e) = self.0.resolve_send_blocking(msg) {
                        log::error!("event_bus dropped: {e}");
                    }
                }
            }

            if let Some(r) = &event_bus.robot.raw_gpio {
                if let Err(e) = r.subscribe(p.0, Box::new(Sub(event_bus.clone()))) {
                    log::error!("Failed to subscribe to gpio pin: {e}");
                }
            }
        }

        _ => todo!(),
    }
}

fn cleanup_resource(event_bus: &Arc<EventBus>, ty: ConcreteType) {
    match ty {
        #[cfg(all(feature = "roland", feature = "backend"))]
        ConcreteType::TrackSensor(_) | ConcreteType::UltraSensor(_) => (),

        #[cfg(all(feature = "gpio", feature = "backend"))]
        ConcreteType::GpioPin(p) => {
            if let Some(r) = &event_bus.robot.raw_gpio {
                if let Err(e) = r.unsubscribe(p.0) {
                    log::error!("Failed to unsubscribe from gpio pin: {e}");
                }
            }
        }

        _ => todo!(),
    }
}

#[cfg(all(feature = "roland", feature = "backend"))]
fn connect_roland(event_bus: Arc<EventBus>) -> anyhow::Result<()> {
    use std::time::{Duration, Instant};

    use roblib::{event, roland::Roland};
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
            let (ty, id, sub) = match rx.blocking_recv() {
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
                            id,
                            interval,
                            next: Instant::now() + interval,
                        });
                    }
                    Unsubscribe => {
                        // TODO:
                        //ultra_subs = track_subs.saturating_sub(1)
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
            next_ultra.next - now - roblib::roland::backend::constants::ultra_sensor::BLAST_DURATION
        } else {
            MAX_WAIT
        };

        if track_subs > 0 {
            if let Some((track_index, val)) = roland.poll_tracksensor(Some(poll_dur))? {
                track_sensor_state[track_index] = val;
                event_bus.resolve_send_blocking((
                    ConcreteType::TrackSensor(event::TrackSensor),
                    ConcreteValue::TrackSensor(track_sensor_state),
                ))?;
            }
        } else {
            std::thread::sleep(poll_dur);
        }

        if let Some(next_ultra) = next_ultra {
            let v = roland.ultra_sensor()?;
            event_bus.send(
                (
                    ConcreteType::UltraSensor(event::UltraSensor(next_ultra.interval)),
                    ConcreteValue::UltraSensor(v),
                ),
                &next_ultra.id,
            )?;
            next_ultra.next += next_ultra.interval;
        }

        // TODO: batching
    }
}
