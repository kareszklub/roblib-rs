use roblib::event::{ConcreteType, ConcreteValue};
use std::{collections::HashMap, sync::Arc};
use sub::EventSub;
use tokio::sync::RwLock;

use crate::transports::{self, SubscriptionId};

/// another channel to handle changes to subscriptions
/// sent by the transport layer, received by the event bus sender workers
pub mod sub {
    use crate::transports::SubscriptionId;
    use tokio::sync::broadcast::{Receiver, Sender};

    #[derive(Debug, Clone)]
    pub enum EventSub {
        Subscribe,
        Unsubscribe,
    }
    /// (event, true for sub, false for unsub)
    pub type Item = (roblib::event::ConcreteType, SubscriptionId, EventSub);
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

    pub async fn resolve_send(&self, event: (ConcreteType, ConcreteValue)) -> anyhow::Result<()> {
        let clients = self.clients.read().await;
        let Some(v) = clients.get(&event.0) else{
            return Ok(());
        };
        self.send_all(event, v)
    }

    pub fn resolve_send_blocking(
        &self,
        event: (ConcreteType, ConcreteValue),
    ) -> anyhow::Result<()> {
        let clients = self.clients.blocking_read();
        let Some(v) = clients.get(&event.0) else{
            return Ok(());
        };
        self.send_all(event, v)
    }

    fn send_all(
        &self,
        event: (ConcreteType, ConcreteValue),
        clients: &Vec<SubscriptionId>,
    ) -> anyhow::Result<()> {
        for client in clients {
            match client {
                SubscriptionId::Udp(addr, id) => {
                    self.bus_udp
                        .send((event.0.clone(), event.1.clone(), (*addr, *id)))?
                }
            };
        }
        Ok(())
    }
}

/// hook up all the "inputs" (backends) to the event bus
pub(super) async fn connect(event_bus: EventBus) {
    let event_bus = Arc::new(event_bus);

    #[cfg(all(feature = "roland", feature = "backend"))]
    if event_bus.robot.roland.is_some() {
        let robot = event_bus.robot.clone();
        std::thread::spawn(move || connect_roland(robot));
    }

    // handle client subscription state changes
    // TODO: on client disconnect, unsubscribe them from all events
    while let Ok(sub) = event_bus.robot.sub.subscribe().recv().await {
        let mut clients = event_bus.clients.write().await;
        let v = clients.entry(sub.0.clone()).or_default();
        match sub.2 {
            EventSub::Subscribe => {
                if v.len() == 0 {
                    // setup resource
                    match sub.0 {
                        #[cfg(all(feature = "roland", feature = "backend"))]
                        ConcreteType::TrackSensor(_) => todo!(),
                        #[cfg(all(feature = "roland", feature = "backend"))]
                        ConcreteType::UltraSensor(_) => todo!(),
                        #[cfg(all(feature = "gpio", feature = "backend"))]
                        ConcreteType::GpioPin(p) => {
                            struct Sub(Arc<EventBus>);
                            impl roblib::gpio::event::Subscriber for Sub {
                                fn handle(&self, event: roblib::gpio::event::Event) {
                                    let msg = match event {
                                        roblib::gpio::event::Event::PinChanged(pin, value) => (
                                            ConcreteType::GpioPin(roblib::gpio::event::GpioPin(
                                                pin,
                                            )),
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
                        _ => (),
                    }
                }
                v.push(sub.1);
            }
            EventSub::Unsubscribe => {
                if v.len() == 0 {
                    log::warn!("Tried to unsubscribe from empty event");
                    continue;
                }
                let Some(i) = v.iter().position(|x| x == &sub.1) else {
                    log::warn!("Tried to unsubscribe but was never subscribed");
                    continue;
                };
                v.remove(i);

                if v.len() == 0 {
                    // cleanup resource
                    match sub.0 {
                        #[cfg(all(feature = "roland", feature = "backend"))]
                        ConcreteType::TrackSensor(_) => todo!(),
                        #[cfg(all(feature = "roland", feature = "backend"))]
                        ConcreteType::UltraSensor(_) => todo!(),
                        #[cfg(all(feature = "gpio", feature = "backend"))]
                        ConcreteType::GpioPin(p) => {
                            if let Some(r) = &event_bus.robot.raw_gpio {
                                if let Err(e) = r.unsubscribe(p.0) {
                                    log::error!("Failed to unsubscribe from gpio pin: {e}");
                                }
                            }
                        }
                        _ => (),
                    }
                }
            }
        }
    }
    log::error!("event_bus_sub dropped");
}

#[cfg(all(feature = "roland", feature = "backend"))]
fn connect_roland(robot: Arc<crate::Backends>) {
    let mut track_subs = 0u32;
    let mut ultra_subs = 0u32;
    let mut rx = robot.sub.subscribe();
    loop {
        if track_subs + ultra_subs == 0 {
            // ch.recv() ...
        } else {
            while track_subs + ultra_subs > 0 {
                while let Ok(sub) = rx.try_recv() {
                    // update subs
                    match sub.0 {
                        ConcreteType::TrackSensor(_) => todo!(),
                        ConcreteType::UltraSensor(_) => todo!(),
                        _ => (),
                    }
                }

                track_subs += 1;

                if track_subs > 0 {
                    // track_sensor
                }

                if ultra_subs > 0 {
                    // ultra_sensor
                }
            }
        }
    }
}
