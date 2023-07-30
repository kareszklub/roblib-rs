use crate::cmd::Command;
use roblib_macro::Command;

#[derive(Command, serde::Serialize, serde::Deserialize, Debug)]
pub struct MoveRobot(pub f64, pub f64);
impl Command for MoveRobot {
    const PREFIX: char = 'm';
    type Return = ();
}

#[derive(Command, serde::Serialize, serde::Deserialize, Debug)]
pub struct MoveRobotByAngle(pub f64, pub f64);
impl Command for MoveRobotByAngle {
    const PREFIX: char = 'M';
    type Return = ();
}

#[derive(Command, serde::Serialize, serde::Deserialize, Debug)]
pub struct StopRobot;
impl Command for StopRobot {
    const PREFIX: char = 's';
    type Return = ();
}

#[derive(Command, serde::Serialize, serde::Deserialize, Debug)]
pub struct Led(pub bool, pub bool, pub bool);
impl Command for Led {
    const PREFIX: char = 'l';
    type Return = ();
}

#[derive(Command, serde::Serialize, serde::Deserialize, Debug)]
pub struct RolandServo(pub f64);
impl Command for RolandServo {
    const PREFIX: char = 'a';
    type Return = ();
}

#[derive(Command, serde::Serialize, serde::Deserialize, Debug)]
pub struct Buzzer(pub f64);
impl Command for Buzzer {
    const PREFIX: char = 'b';
    type Return = ();
}

#[derive(Command, serde::Serialize, serde::Deserialize, Debug)]
pub struct TrackSensor;
impl Command for TrackSensor {
    const PREFIX: char = 't';
    type Return = [bool; 4];
}

// TODO: return Option<f64>
#[derive(Command, serde::Serialize, serde::Deserialize, Debug)]
pub struct UltraSensor;
impl Command for UltraSensor {
    const PREFIX: char = 'u';
    type Return = f64;
}
