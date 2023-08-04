use roblib_macro::Event;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub enum Event {
    PinChanged(u8, bool),
}

#[derive(Event, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct GpioPin(pub u8);
impl crate::event::Event for GpioPin {
    const NAME: &'static str = "pin";
    type Item = bool;
}
