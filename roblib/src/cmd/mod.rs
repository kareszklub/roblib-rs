use roblib_macro::Command;
use serde::{de::DeserializeOwned, Serialize};

pub mod concrete;

use crate::event;

#[cfg(feature = "roland")]
pub use crate::roland::cmd::*;

#[cfg(feature = "gpio")]
pub use crate::gpio::cmd::*;

#[cfg(feature = "camloc")]
pub use crate::camloc::cmd::*;

pub use self::concrete::Concrete;

pub trait Command: Serialize + DeserializeOwned + Into<Concrete> + From<Concrete> {
    const PREFIX: char;
    type Return: Serialize + DeserializeOwned;
}

pub const SEPARATOR: char = ' ';

pub const fn has_return<C: Command>() -> bool {
    std::mem::size_of::<C::Return>() != 0
}

#[derive(Command, serde::Serialize, serde::Deserialize, Debug)]
pub struct Subscribe(pub event::ConcreteType);
impl Command for Subscribe {
    const PREFIX: char = '+';
    type Return = ();
}
#[derive(Command, serde::Serialize, serde::Deserialize, Debug)]
pub struct Unsubscribe(pub event::ConcreteType);
impl Command for Unsubscribe {
    const PREFIX: char = '-';
    type Return = ();
}

#[derive(Command, serde::Serialize, serde::Deserialize, Debug)]
pub struct Nop;
impl Command for Nop {
    const PREFIX: char = '0';
    type Return = ();
}

#[derive(Command, serde::Serialize, serde::Deserialize, Debug)]
pub struct GetUptime;
impl Command for GetUptime {
    const PREFIX: char = 'U';
    type Return = std::time::Duration;
}
