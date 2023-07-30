use crate::cmd::Command;
use roblib_macro::Command;

#[derive(Command, serde::Serialize, serde::Deserialize, Debug)]
pub struct GetPosition;
impl Command for GetPosition {
    const PREFIX: char = 'P';
    type Return = Option<super::Position>;
}
