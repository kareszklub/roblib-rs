use parsing::{Readable, Writable};
use roblib_macro::Command;

pub mod concrete;
pub mod parsing;

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

pub const SEPARATOR: char = ' ';

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
