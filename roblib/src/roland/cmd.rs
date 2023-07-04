use crate::cmd::Command;
use roblib_macro::Command;

#[derive(Command, serde::Serialize, serde::Deserialize)]
pub struct MoveRobot(pub f64, pub f64);
impl Command for MoveRobot {
    const PREFIX: char = 'm';
    type Return = ();
}

#[derive(Command, serde::Serialize, serde::Deserialize)]
pub struct MoveRobotByAngle(pub f64, pub f64);
impl Command for MoveRobotByAngle {
    const PREFIX: char = 'M';
    type Return = ();
}

#[derive(Command, serde::Serialize, serde::Deserialize)]
pub struct StopRobot;
impl Command for StopRobot {
    const PREFIX: char = 's';
    type Return = ();
}

#[derive(Command, serde::Serialize, serde::Deserialize)]
pub struct Led(pub bool, pub bool, pub bool);
impl Command for Led {
    const PREFIX: char = 'l';
    type Return = ();
}

#[derive(Command, serde::Serialize, serde::Deserialize)]
pub struct ServoAbsolute(pub f64);
impl Command for ServoAbsolute {
    const PREFIX: char = 'a';
    type Return = ();
}

#[derive(Command, serde::Serialize, serde::Deserialize)]
pub struct Buzzer(pub f64);
impl Command for Buzzer {
    const PREFIX: char = 'b';
    type Return = ();
}

#[derive(Command, serde::Serialize, serde::Deserialize)]
pub struct TrackSensor;
impl Command for TrackSensor {
    const PREFIX: char = 't';
    type Return = [bool; 4];
}

#[derive(Command, serde::Serialize, serde::Deserialize)]
pub struct UltraSensor;
impl Command for UltraSensor {
    const PREFIX: char = 'u';
    type Return = f64;
}
