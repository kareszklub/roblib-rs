use crate::{transports::Transport, Robot};
use anyhow::Result;
use roblib::cmd;

impl<T: Transport> roblib::roland::Roland for Robot<T> {
    fn drive(&self, left: f64, right: f64) -> Result<()> {
        if !(-1. ..=1.).contains(&left) || !(-1. ..=1.).contains(&right) {
            log::warn!("Drive values are now [-1, 1] not [-100, 100]");
        }
        self.transport.cmd(cmd::MoveRobot(left, right))
    }

    fn drive_by_angle(&self, angle: f64, speed: f64) -> Result<()> {
        if !(-1. ..=1.).contains(&speed) {
            log::warn!("Drive values are now [-1, 1] not [-100, 100]");
        }
        self.transport.cmd(cmd::MoveRobotByAngle(angle, speed))
    }

    fn led(&self, r: bool, g: bool, b: bool) -> Result<()> {
        self.transport.cmd(cmd::Led(r, g, b))
    }

    fn servo(&self, degree: f64) -> Result<()> {
        self.transport.cmd(cmd::RolandServo(degree))
    }

    fn buzzer(&self, pw: f64) -> Result<()> {
        self.transport.cmd(cmd::Buzzer(pw))
    }

    fn track_sensor(&self) -> Result<[bool; 4]> {
        self.transport.cmd(cmd::TrackSensor)
    }

    fn ultra_sensor(&self) -> Result<f64> {
        self.transport.cmd(cmd::UltraSensor)
    }

    fn stop(&self) -> Result<()> {
        self.transport.cmd(cmd::StopRobot)
    }
}
