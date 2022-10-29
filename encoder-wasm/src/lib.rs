use roblib::cmd::{binary_opts as opts, BincodeOptions, Cmd};
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub fn move_robot(l: i8, r: i8) -> Vec<u8> {
    opts().serialize(&Cmd::MoveRobot(l, r)).unwrap()
}

#[wasm_bindgen]
pub fn move_robot_by_angle(angle: f64, speed: i8) -> Vec<u8> {
    opts()
        .serialize(&Cmd::MoveRobotByAngle(angle, speed, None))
        .unwrap()
}
#[wasm_bindgen]
pub fn move_robot_by_angle_with_leds(angle: f64, speed: i8, r: bool, g: bool, b: bool) -> Vec<u8> {
    opts()
        .serialize(&Cmd::MoveRobotByAngle(angle, speed, Some((r, g, b))))
        .unwrap()
}

#[wasm_bindgen]
pub fn stop_robot() -> Vec<u8> {
    opts().serialize(&Cmd::StopRobot).unwrap()
}

#[wasm_bindgen]
pub fn led(r: bool, g: bool, b: bool) -> Vec<u8> {
    opts().serialize(&Cmd::Led(r, g, b)).unwrap()
}

#[wasm_bindgen]
pub fn servo_absolute(angle: i8) -> Vec<u8> {
    opts().serialize(&Cmd::ServoAbsolute(angle)).unwrap()
}

#[wasm_bindgen]
pub fn buzzer(pw: f64) -> Vec<u8> {
    opts().serialize(&Cmd::Buzzer(pw)).unwrap()
}

#[wasm_bindgen]
pub fn track_sensor() -> Vec<u8> {
    opts().serialize(&Cmd::TrackSensor).unwrap()
}

#[wasm_bindgen]
pub fn set_pin(pin: u8, value: bool) -> Vec<u8> {
    opts().serialize(&Cmd::SetPin(pin, value)).unwrap()
}

#[wasm_bindgen]
pub fn set_pwm(pin: u8, hz: f64, cycle: f64) -> Vec<u8> {
    opts().serialize(&Cmd::SetPwm(pin, hz, cycle)).unwrap()
}

#[wasm_bindgen]
pub fn servo_basic(pin: u8, angle: i8) -> Vec<u8> {
    opts().serialize(&Cmd::ServoBasic(pin, angle)).unwrap()
}

#[wasm_bindgen]
pub fn get_time() -> Vec<u8> {
    opts().serialize(&Cmd::GetTime).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode() {
        let s = "l 1 0 1";
        let bytes = opts()
            .serialize(&<Cmd as std::str::FromStr>::from_str(s).unwrap())
            .unwrap();
        let result: Cmd = opts().deserialize(&bytes).unwrap();
        let result = result.to_string();
        assert_eq!(result, s);
    }

    #[test]
    fn test_size() {
        let b = opts().serialize(&Cmd::MoveRobot(1, 2)).unwrap();
        assert_eq!(b.len(), 3);

        let b = opts()
            .serialize(&Cmd::MoveRobotByAngle(
                std::f64::consts::PI - 100.0,
                1,
                Some((true, true, true)),
            ))
            .unwrap();
        assert_eq!(b.len(), 14);
    }
}
