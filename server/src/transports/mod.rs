use std::net::SocketAddr;

// pub mod http;
// pub mod tcp;
pub mod udp;
// pub mod ws;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SubscriptionId {
    Udp(SocketAddr, u32),
}
