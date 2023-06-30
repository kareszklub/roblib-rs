use roblib_macro::Command;
use roblib_parsing::{Readable, Writable};

pub mod concrete;

use crate::event;
#[cfg(feature = "roland")]
pub use crate::roland::cmd::*;

#[cfg(feature = "gpio")]
pub use crate::gpio::cmd::*;

#[cfg(feature = "camloc")]
pub use crate::camloc::cmd::*;

pub use self::concrete::Concrete;

pub trait Command: Readable + Writable + Into<Concrete> + From<Concrete> + 'static {
    const PREFIX: char;
    type Return: Readable;
}

pub fn has_return<C: Command>() -> bool {
    std::any::TypeId::of::<C::Return>() != std::any::TypeId::of::<()>()
}

extern crate self as roblib;

#[derive(Command)]
pub struct Subscribe(event::Concrete);
impl Command for Subscribe {
    const PREFIX: char = 'a';
    type Return = std::time::Duration;
}
#[derive(Command)]
pub struct Unsubscribe(event::Concrete);
impl Command for Unsubscribe {
    const PREFIX: char = 'A';
    type Return = std::time::Duration;
}

#[derive(Command)]
pub struct Nop;
impl Command for Nop {
    const PREFIX: char = 'n';
    type Return = ();
}

#[derive(Command)]
pub struct GetUptime;
impl Command for GetUptime {
    const PREFIX: char = 'U';
    type Return = std::time::Duration;
}
