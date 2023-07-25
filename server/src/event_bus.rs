use roblib::event::{ConcreteType, ConcreteValue};
use std::sync::Arc;
use tokio::sync::broadcast::{channel, Receiver, Sender};

pub type Item = (roblib::event::ConcreteType, roblib::event::ConcreteValue);
pub type Tx = Sender<Item>;
pub type Rx = Receiver<Item>;

pub struct EventBus {
    pub tx: Tx,
    pub rx: Rx,

    pub sub_tx: sub::Tx,
    pub sub_rx: sub::Rx,
}

pub fn init() -> EventBus {
    let (tx, rx) = channel(1);
    let (sub_tx, sub_rx) = channel(1);
    EventBus {
        rx,
        tx,
        sub_rx,
        sub_tx,
    }
}

/// hook up all the "inputs" (backends) to the event bus
pub(super) async fn connect(robot: Arc<crate::Backends>) {
    while let Ok(sub) = robot.event_bus.sub_rx.resubscribe().recv().await {
        match sub {
            #[cfg(all(feature = "gpio", feature = "backend"))]
            (ConcreteType::GpioPin(p), true) => {
                struct Sub(Tx);
                impl roblib::gpio::event::Subscriber for Sub {
                    fn handle(&self, event: roblib::gpio::event::Event) {
                        if let Err(e) = self.0.send(match event {
                            roblib::gpio::event::Event::PinChanged(pin, value) => (
                                ConcreteType::GpioPin(roblib::gpio::event::GpioPin(pin)),
                                ConcreteValue::GpioPin(value),
                            ),
                        }) {
                            log::error!("event_bus dropped: {e}");
                        }
                    }
                }
                if let Some(r) = &robot.raw_gpio {
                    if let Err(e) = r.subscribe(p.0, Box::new(Sub(robot.event_bus.tx.clone()))) {
                        log::error!("Failed to subscribe to gpio pin: {e}");
                    }
                }
            }
            #[cfg(all(feature = "gpio", feature = "backend"))]
            (ConcreteType::GpioPin(p), false) => {
                if let Some(r) = &robot.raw_gpio {
                    if let Err(e) = r.unsubscribe(p.0) {
                        log::error!("Failed to unsubscribe from gpio pin: {e}");
                    }
                }
            }

            _ => (),
        }
    }
    log::error!("event_bus_sub dropped");
}

/// another channel to handle changes to subscriptions
/// sent by the transport layer, received by the event bus sender workers
pub mod sub {
    use tokio::sync::broadcast::{Receiver, Sender};
    /// (event, true for sub, false for unsub)
    pub type Item = (roblib::event::ConcreteType, bool);
    pub type Tx = Sender<Item>;
    pub type Rx = Receiver<Item>;
}
