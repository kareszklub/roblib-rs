use crate::cmd::Command;
use roblib_macro::Command;

#[derive(Command, serde::Serialize, serde::Deserialize, Debug)]
pub struct PinMode(pub u8, pub super::Mode);
impl Command for PinMode {
    const PREFIX: char = 'p';
    type Return = ();
}

#[derive(Command, serde::Serialize, serde::Deserialize, Debug)]
pub struct ReadPin(pub u8);
impl Command for ReadPin {
    const PREFIX: char = 'r';
    type Return = bool;
}

#[derive(Command, serde::Serialize, serde::Deserialize, Debug)]
pub struct WritePin(pub u8, pub bool);
impl Command for WritePin {
    const PREFIX: char = 'w';
    type Return = ();
}

#[derive(Command, serde::Serialize, serde::Deserialize, Debug)]
pub struct Pwm(pub u8, pub f64, pub f64);
impl Command for Pwm {
    const PREFIX: char = 'W';
    type Return = ();
}

#[derive(Command, serde::Serialize, serde::Deserialize, Debug)]
pub struct Servo(pub u8, pub f64);
impl Command for Servo {
    const PREFIX: char = 'V';
    type Return = ();
}
