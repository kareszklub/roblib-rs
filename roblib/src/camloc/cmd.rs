use roblib_macro::{Readable, Writable};

use crate::cmd::{
    parsing::{Readable, Writable},
    Command,
};

#[derive(Readable, Writable)]
pub struct GetPosition;
impl Command for GetPosition {
    const PREFIX: char = 'P';
    type Return = Option<super::Position>;
}
