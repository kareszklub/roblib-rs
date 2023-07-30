// pub mod http;
pub mod tcp;
pub mod udp;
// pub mod ws;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SubscriptionId {
    Tcp(tcp::Id, tcp::SubId),
    Udp(udp::Id, udp::SubId),
}

impl SubscriptionId {
    pub fn same_client(&self, other: &Self) -> bool {
        match (self, other) {
            (SubscriptionId::Tcp(addr1, _), SubscriptionId::Tcp(addr2, _)) => *addr1 == *addr2,
            (SubscriptionId::Tcp(_, _), _) => false,

            (SubscriptionId::Udp(addr1, _), SubscriptionId::Udp(addr2, _)) => *addr1 == *addr2,
            (SubscriptionId::Udp(_, _), _) => false,
        }
    }
}

// #[derive(Debug, PartialEq, Eq, Clone)]
// pub enum ClientId {
//     Udp(udp::Id),
// }

// impl From<SubscriptionId> for ClientId {
//     fn from(value: SubscriptionId) -> Self {
//         match value {
//             SubscriptionId::Udp(id, _) => Self::Udp(id),
//         }
//     }
// }
