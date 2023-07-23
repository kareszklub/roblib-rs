use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub trait Event: Serialize + DeserializeOwned + Into<ConcreteType> + From<ConcreteType> {
    type Item: Serialize + DeserializeOwned;
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub enum ConcreteType {
    #[cfg(feature = "gpio")]
    GpioPin(crate::gpio::event::GpioPin),

    #[cfg(feature = "camloc")]
    CamlocConnect(crate::camloc::event::CamlocConnect),
    #[cfg(feature = "camloc")]
    CamlocDisconnect(crate::camloc::event::CamlocDisconnect),
    #[cfg(feature = "camloc")]
    CamlocPosition(crate::camloc::event::CamlocPosition),
    #[cfg(feature = "camloc")]
    CamlocInfoUpdate(crate::camloc::event::CamlocInfoUpdate),

    None,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ConcreteValue {
    #[cfg(feature = "gpio")]
    GpioPin(<crate::gpio::event::GpioPin as Event>::Item),

    #[cfg(feature = "camloc")]
    CamlocConnect(<crate::camloc::event::CamlocConnect as Event>::Item),
    #[cfg(feature = "camloc")]
    CamlocDisconnect(<crate::camloc::event::CamlocDisconnect as Event>::Item),
    #[cfg(feature = "camloc")]
    CamlocPosition(<crate::camloc::event::CamlocPosition as Event>::Item),
    #[cfg(feature = "camloc")]
    CamlocInfoUpdate(<crate::camloc::event::CamlocInfoUpdate as Event>::Item),

    None,
}
