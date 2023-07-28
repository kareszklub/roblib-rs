pub use camloc_server::service::Subscriber;
use camloc_server::{PlacedCamera, Position};
use roblib_macro::Event;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Event, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct CamlocConnect;
impl crate::event::Event for CamlocConnect {
    type Item = (SocketAddr, PlacedCamera);
}

#[derive(Event, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct CamlocDisconnect;
impl crate::event::Event for CamlocDisconnect {
    type Item = SocketAddr;
}

#[derive(Event, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct CamlocPosition;
impl crate::event::Event for CamlocPosition {
    type Item = Position;
}

#[derive(Event, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct CamlocInfoUpdate;
impl crate::event::Event for CamlocInfoUpdate {
    type Item = (SocketAddr, PlacedCamera);
}
