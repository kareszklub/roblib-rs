use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub trait Event: Serialize + DeserializeOwned + Into<Concrete> + From<Concrete> {
    type Item: Serialize + DeserializeOwned;
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub enum Concrete {
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
