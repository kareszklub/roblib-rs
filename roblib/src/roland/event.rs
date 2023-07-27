use std::time::Duration;

use roblib_macro::Event;
use serde::{Deserialize, Serialize};

#[derive(Event, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
pub struct TrackSensor;
impl crate::event::Event for TrackSensor {
    type Item = [bool; 4];
}

#[derive(Event, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
pub struct UltraSensor(pub Duration);
impl crate::event::Event for UltraSensor {
    type Item = f64;
}
