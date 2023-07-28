// pub mod http;
// pub mod tcp;
pub mod udp;
// pub mod ws;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SubscriptionId {
    Udp(udp::Id, udp::SubId),
}

impl SubscriptionId {
    pub fn same_client(&self, other: &Self) -> bool {
        match (self, other) {
            (SubscriptionId::Udp(addr1, _), SubscriptionId::Udp(addr2, _)) => addr1 == addr2,
            (SubscriptionId::Udp(_, _), _) => false,

            _ => false,
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
