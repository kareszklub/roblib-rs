#[macro_use]
extern crate log;

pub mod logger;

pub mod http;
pub mod tcp;
pub mod udp;
pub mod ws;

use std::time::{Duration, Instant};

pub use anyhow::Result;

pub use roblib;

use roblib::{
    cmd::{self, parsing::Readable, Command},
    RoblibRobot,
};

pub trait RemoteRobotTransport {
    fn cmd<C: Command>(&self, cmd: C) -> Result<C::Return>
    where
        C::Return: Readable;

    fn measure_latency(&self) -> Result<Duration> {
        let start = Instant::now();
        self.cmd(cmd::GetUptime)?;
        Ok(Instant::now() - start)
    }

    fn get_server_uptime(&self) -> Result<Duration> {
        self.cmd(cmd::GetUptime)
    }
}

pub struct Robot<T> {
    pub transport: T,
}

impl<T: RemoteRobotTransport> Robot<T> {
    pub fn new(transport: T) -> Self {
        Self { transport }
    }
}

impl<T: RemoteRobotTransport> RoblibRobot for Robot<T> {
    fn nop(&self) -> anyhow::Result<()> {
        self.transport.cmd(cmd::Nop)
    }

    fn get_uptime(&self) -> anyhow::Result<Duration> {
        self.transport.cmd(cmd::GetUptime)
    }
}

#[cfg(feature = "roland")]
impl<T: RemoteRobotTransport> roblib::roland::Roland for Robot<T> {
    fn drive(&self, left: f64, right: f64) -> Result<roblib::roland::DriveResult> {
        if !(-1. ..=1.).contains(&left) || !(-1. ..=1.).contains(&right) {
            warn!("Drive values are now [-1, 1] not [-100, 100]");
        }
        self.transport.cmd(cmd::MoveRobot(left, right))
    }

    fn drive_by_angle(&self, angle: f64, speed: f64) -> Result<roblib::roland::DriveResult> {
        if !(-1. ..=1.).contains(&speed) {
            warn!("Drive values are now [-1, 1] not [-100, 100]");
        }
        self.transport.cmd(cmd::MoveRobotByAngle(angle, speed))
    }

    fn led(&self, r: bool, g: bool, b: bool) -> Result<()> {
        self.transport.cmd(cmd::Led(r, g, b))
    }

    fn servo(&self, degree: f64) -> Result<()> {
        self.transport.cmd(cmd::ServoAbsolute(degree))
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

#[cfg(feature = "gpio")]
impl<T: RemoteRobotTransport> roblib::gpio::Gpio for Robot<T> {
    fn read_pin(&self, pin: u8) -> Result<bool> {
        self.transport.cmd(cmd::ReadPin(pin))
    }

    fn set_pin(&self, pin: u8, value: bool) -> Result<()> {
        self.transport.cmd(cmd::SetPin(pin, value))
    }

    fn pwm(&self, pin: u8, hz: f64, cycle: f64) -> Result<()> {
        self.transport.cmd(cmd::SetPwm(pin, hz, cycle))
    }

    fn servo(&self, pin: u8, degree: f64) -> Result<()> {
        self.transport.cmd(cmd::ServoBasic(pin, degree))
    }
}

#[cfg(feature = "camloc")]
impl<T: RemoteRobotTransport> Robot<T> {
    pub fn get_position(&self) -> Result<Option<roblib::camloc::Position>> {
        self.transport.cmd(cmd::GetPosition)
    }
}
