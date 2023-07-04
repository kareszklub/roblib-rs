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

pub trait Command:
    Serialize + DeserializeOwned + Into<Concrete> + From<Concrete> + 'static
{
    const PREFIX: char;
    type Return: Serialize + DeserializeOwned + 'static;
}

pub const SEPARATOR: char = ' ';

pub const fn has_return<C: Command>() -> bool {
    std::mem::size_of::<C::Return>() != 0
}

#[derive(Command, serde::Serialize, serde::Deserialize)]
pub struct Subscribe(event::Concrete);
impl Command for Subscribe {
    const PREFIX: char = '+';
    type Return = std::time::Duration;
}
#[derive(Command, serde::Serialize, serde::Deserialize)]
pub struct Unsubscribe(event::Concrete);
impl Command for Unsubscribe {
    const PREFIX: char = '-';
    type Return = std::time::Duration;
}

#[derive(Command, serde::Serialize, serde::Deserialize)]
pub struct Nop;
impl Command for Nop {
    const PREFIX: char = 'n';
    type Return = ();
}

#[derive(Command, serde::Serialize, serde::Deserialize)]
pub struct GetUptime;
impl Command for GetUptime {
    const PREFIX: char = 'U';
    type Return = std::time::Duration;
}
