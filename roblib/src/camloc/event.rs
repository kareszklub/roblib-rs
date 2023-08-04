use camloc_server::{PlacedCamera, Position};
use roblib_macro::Event;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Event, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct CamlocConnect;
impl crate::event::Event for CamlocConnect {
    const NAME: &'static str = "camloc_connect";
    type Item = (SocketAddr, PlacedCamera);
}

#[derive(Event, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct CamlocDisconnect;
impl crate::event::Event for CamlocDisconnect {
    const NAME: &'static str = "camloc_disconnect";
    type Item = SocketAddr;
}

#[derive(Event, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct CamlocPosition;
impl crate::event::Event for CamlocPosition {
    const NAME: &'static str = "position";
    type Item = Position;
}

#[derive(Event, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct CamlocInfoUpdate;
impl crate::event::Event for CamlocInfoUpdate {
    const NAME: &'static str = "camloc_info_update";
    type Item = (SocketAddr, PlacedCamera);
}
