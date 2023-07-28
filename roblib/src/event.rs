use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[cfg(feature = "roland")]
pub use crate::roland::event::*;

#[cfg(feature = "gpio")]
pub use crate::gpio::event::*;

#[cfg(feature = "camloc")]
pub use crate::camloc::event::*;

pub trait Event: Serialize + DeserializeOwned + Into<ConcreteType> + From<ConcreteType> {
    type Item: Serialize + DeserializeOwned;
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum ConcreteType {
    #[cfg(feature = "roland")]
    TrackSensor(crate::roland::event::TrackSensor),
    #[cfg(feature = "roland")]
    UltraSensor(crate::roland::event::UltraSensor),

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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ConcreteValue {
    #[cfg(feature = "roland")]
    TrackSensor(<crate::roland::event::TrackSensor as Event>::Item),
    #[cfg(feature = "roland")]
    UltraSensor(<crate::roland::event::UltraSensor as Event>::Item),

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
