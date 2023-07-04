use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub trait Event: Serialize + DeserializeOwned + Into<Concrete> + From<Concrete> + 'static {
    type Item: Serialize + DeserializeOwned + 'static;
}

#[derive(Serialize, Deserialize)]
pub enum Concrete {
    #[cfg(feature = "gpio")]
    GpioPin(crate::gpio::event::GpioPin),

    #[cfg(feature = "camloc")]
    Camloc(crate::camloc::event::Event),
}
