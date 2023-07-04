use crate::cmd::Command;
use roblib_macro::Command;

#[derive(Command, serde::Serialize, serde::Deserialize)]
pub struct ReadPin(pub u8);
impl Command for ReadPin {
    const PREFIX: char = 'r';
    type Return = bool;
}

#[derive(Command, serde::Serialize, serde::Deserialize)]
pub struct SetPin(pub u8, pub bool);
impl Command for SetPin {
    const PREFIX: char = 'p';
    type Return = ();
}

#[derive(Command, serde::Serialize, serde::Deserialize)]
pub struct SetPwm(pub u8, pub f64, pub f64);
impl Command for SetPwm {
    const PREFIX: char = 'w';
    type Return = ();
}

#[derive(Command, serde::Serialize, serde::Deserialize)]
pub struct ServoBasic(pub u8, pub f64);
impl Command for ServoBasic {
    const PREFIX: char = 'V';
    type Return = ();
}
