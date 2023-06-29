use roblib_macro::Command;

use crate::cmd::{
    parsing::{Readable, Writable},
    Command,
};

#[derive(Command)]
pub struct GetPosition;
impl Command for GetPosition {
    const PREFIX: char = 'P';
    type Return = Option<super::Position>;
}
