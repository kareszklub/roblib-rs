use crate::cmd::Command;
use roblib_macro::Command;

extern crate self as roblib;

#[derive(Command)]
pub struct GetPosition;
impl Command for GetPosition {
    const PREFIX: char = 'P';
    type Return = Option<super::Position>;
}
