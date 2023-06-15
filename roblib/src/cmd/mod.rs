use parsing::{Readable, Writable};
use roblib_macro::{Readable, Writable};

pub mod parsing;

#[cfg(feature = "roland")]
pub use crate::roland::cmd::*;

#[cfg(feature = "gpio")]
pub use crate::gpio::cmd::*;

#[cfg(feature = "camloc")]
pub use crate::camloc::cmd::*;

pub trait Command: Readable + Writable {
    const PREFIX: char;
    type Return;
}

pub const SEPARATOR: char = ' ';

#[derive(Readable, Writable)]
pub struct Nop;
impl Command for Nop {
    const PREFIX: char = 'n';
    type Return = ();
}

#[derive(Readable, Writable)]
pub struct GetUptime;
impl Command for GetUptime {
    const PREFIX: char = 'U';
    type Return = std::time::Duration;
}
