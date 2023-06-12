use std::{collections::HashSet, net::SocketAddr};

use roblib::camloc::server::service::LocationServiceHandle;

pub struct Camloc {
    pub service: LocationServiceHandle,
    pub ws_subscribers: HashSet<SocketAddr>,
}

impl Camloc {
    pub fn new(service: LocationServiceHandle) -> Self {
        Self {
            ws_subscribers: HashSet::new(),
            service,
        }
    }
}
