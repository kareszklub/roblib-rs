use serde::{de::DeserializeOwned, Serialize};

pub mod concrete;
pub use concrete::{ConcreteType, ConcreteValue};

#[cfg(feature = "roland")]
pub use crate::roland::event::*;

#[cfg(feature = "gpio")]
pub use crate::gpio::event::*;

#[cfg(feature = "camloc")]
pub use crate::camloc::event::*;

pub trait Event:
    Serialize + DeserializeOwned //
    + Into<ConcreteType> + From<ConcreteType> //
    + Clone //
    + Send + Sync + 'static
{
    const NAME: &'static str;
    type Item:
        Serialize + DeserializeOwned //
        + Clone //
        + Send + Sync + 'static;
}
