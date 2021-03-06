use crate::robot::Robot;
use roblib_shared::cmd::{get_time, Cmd};
use std::str::FromStr;

pub fn exec(cmd: &Cmd, robot: &Robot) -> String {
    match cmd {
        Cmd::TrackSensor => {
            let res = robot.track_sensor().map(|x| x as u8);
            format!("{},{},{},{}", res[0], res[1], res[2], res[3])
        }
        Cmd::GetTime => format!("{:.3}", get_time()),
        _ => {
            match cmd {
                // commands that don't have a return type
                Cmd::MoveRobot(left, right) => robot.move_robot(*left, *right),
                Cmd::StopRobot => robot.stop_robot(),
                Cmd::Led(r, g, b) => robot.led(*r, *g, *b),
                Cmd::ServoAbsolute(degree) => robot.servo_absolute(*degree),
                Cmd::Buzzer(pw) => robot.buzzer(*pw),
                _ => unreachable!(),
            };
            "OK".to_string()
        }
    }
}

pub fn exec_str(s: &str, robot: &Robot) -> String {
    match Cmd::from_str(s) {
        Ok(cmd) => exec(&cmd, robot),
        Err(err) => err.to_string(),
    }
}
