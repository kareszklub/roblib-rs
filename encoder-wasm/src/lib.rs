use std::time::Duration;

use bincode::{DefaultOptions, Options};
use roblib::{cmd, gpio::Mode};
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub fn drive(id: u32, l: f64, r: f64) -> Vec<u8> {
    Options::serialize(
        DefaultOptions::new(),
        &(id, cmd::Concrete::MoveRobot(cmd::MoveRobot(l, r))),
    )
    .unwrap()
}

#[wasm_bindgen]
pub fn move_robot_by_angle(id: u32, angle: f64, speed: f64) -> Vec<u8> {
    Options::serialize(
        DefaultOptions::new(),
        &(
            id,
            cmd::Concrete::MoveRobotByAngle(cmd::MoveRobotByAngle(angle, speed)),
        ),
    )
    .unwrap()
}

#[wasm_bindgen]
pub fn stop_robot(id: u32) -> Vec<u8> {
    Options::serialize(
        DefaultOptions::new(),
        &(id, cmd::Concrete::StopRobot(cmd::StopRobot)),
    )
    .unwrap()
}

#[wasm_bindgen]
pub fn led(id: u32, r: bool, g: bool, b: bool) -> Vec<u8> {
    Options::serialize(
        DefaultOptions::new(),
        &(id, cmd::Concrete::Led(cmd::Led(r, g, b))),
    )
    .unwrap()
}

#[wasm_bindgen]
pub fn roland_servo(id: u32, angle: f64) -> Vec<u8> {
    Options::serialize(
        DefaultOptions::new(),
        &(id, cmd::Concrete::RolandServo(cmd::RolandServo(angle))),
    )
    .unwrap()
}

#[wasm_bindgen]
pub fn buzzer(id: u32, pw: f64) -> Vec<u8> {
    Options::serialize(
        DefaultOptions::new(),
        &(id, cmd::Concrete::Buzzer(cmd::Buzzer(pw))),
    )
    .unwrap()
}

#[wasm_bindgen]
pub fn track_sensor(id: u32) -> Vec<u8> {
    Options::serialize(
        DefaultOptions::new(),
        &(id, cmd::Concrete::TrackSensor(cmd::TrackSensor)),
    )
    .unwrap()
}

#[wasm_bindgen]
pub fn ultra_sensor(id: u32) -> Vec<u8> {
    Options::serialize(
        DefaultOptions::new(),
        &(id, cmd::Concrete::UltraSensor(cmd::UltraSensor)),
    )
    .unwrap()
}

#[wasm_bindgen]
pub fn read_pin(id: u32, pin: u8) -> Vec<u8> {
    Options::serialize(
        DefaultOptions::new(),
        &(id, cmd::Concrete::ReadPin(cmd::ReadPin(pin))),
    )
    .unwrap()
}

#[wasm_bindgen]
pub fn write_pin(id: u32, pin: u8, value: bool) -> Vec<u8> {
    Options::serialize(
        DefaultOptions::new(),
        &(id, cmd::Concrete::WritePin(cmd::WritePin(pin, value))),
    )
    .unwrap()
}

#[wasm_bindgen]
pub fn set_pwm(id: u32, pin: u8, hz: f64, cycle: f64) -> Vec<u8> {
    Options::serialize(
        DefaultOptions::new(),
        &(id, cmd::Concrete::Pwm(cmd::Pwm(pin, hz, cycle))),
    )
    .unwrap()
}

#[wasm_bindgen]
pub fn servo_basic(id: u32, pin: u8, angle: f64) -> Vec<u8> {
    Options::serialize(
        DefaultOptions::new(),
        &(id, cmd::Concrete::Servo(cmd::Servo(pin, angle))),
    )
    .unwrap()
}

#[wasm_bindgen]
pub fn pin_mode(id: u32, pin: u8, mode: &str) -> Vec<u8> {
    let mode: Mode = match mode {
        "input" => Mode::Input,
        "output" => Mode::Output,
        _ => panic!(r#"Invalid pin mode. Valid options are "input" and "output""#),
    };
    Options::serialize(
        DefaultOptions::new(),
        &(id, cmd::Concrete::PinMode(cmd::PinMode(pin, mode))),
    )
    .unwrap()
}

#[wasm_bindgen]
pub fn get_uptime(id: u32) -> Vec<u8> {
    Options::serialize(
        DefaultOptions::new(),
        &(id, cmd::Concrete::GetUptime(cmd::GetUptime)),
    )
    .unwrap()
}

#[wasm_bindgen]
pub struct Response {
    pub id: u32,
    data: Vec<u8>,
}
#[wasm_bindgen]
impl Response {
    #[wasm_bindgen]
    pub fn get_data(self) -> Vec<u8> {
        self.data
    }
}

#[wasm_bindgen]
pub fn decode_resp(data: Vec<u8>) -> Response {
    let data: (u32, Vec<u8>) = Options::deserialize(DefaultOptions::new(), &data).unwrap();
    Response {
        id: data.0,
        data: data.1,
    }
}

#[wasm_bindgen]
pub fn decode_uptime(data: Vec<u8>) -> u64 {
    let dur: Duration = Options::deserialize(DefaultOptions::new(), &data).unwrap();
    dur.as_secs()
}

#[wasm_bindgen]
pub fn decode_track_sensor(data: Vec<u8>) -> u8 {
    let data: <cmd::TrackSensor as cmd::Command>::Return =
        Options::deserialize(DefaultOptions::new(), &data).unwrap();
    let mut n = 0;
    for (i, b) in data.into_iter().enumerate() {
        if b {
            n = n | 1 << i;
        }
    }
    n
}

#[wasm_bindgen]
pub fn decode_ultra_sensor(data: Vec<u8>) -> <cmd::UltraSensor as cmd::Command>::Return {
    Options::deserialize(DefaultOptions::new(), &data).unwrap()
}

#[wasm_bindgen]
pub fn decode_read_pin(data: Vec<u8>) -> <cmd::ReadPin as cmd::Command>::Return {
    Options::deserialize(DefaultOptions::new(), &data).unwrap()
}
