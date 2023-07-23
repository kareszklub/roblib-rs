use roblib_macro::Event;
use serde::{Deserialize, Serialize};

pub enum Event {
    PinChanged(bool),
}

pub trait Subscriber {
    fn handle(&self, event: Event);
}

#[derive(Event, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct GpioPin(pub u8);
impl crate::event::Event for GpioPin {
    type Item = bool;
}
