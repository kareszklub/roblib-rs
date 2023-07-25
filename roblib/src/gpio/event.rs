use roblib_macro::Event;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub enum Event {
    PinChanged(u8, bool),
}

pub trait Subscriber: Send + Sync {
    fn handle(&self, event: Event);
}

#[derive(Event, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
pub struct GpioPin(pub u8);
impl crate::event::Event for GpioPin {
    type Item = bool;
}
