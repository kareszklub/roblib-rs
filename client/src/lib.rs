#[macro_use]
extern crate log;
pub mod http;
pub mod logger;
pub mod ws;

pub use anyhow::Result;

pub use roblib;

use roblib::{
    cmd::{get_time, Cmd},
    roland::DriveResult,
};

pub trait RemoteRobotTransport {
    fn cmd(&self, cmd: Cmd) -> Result<String>;

    fn measure_latency(&self) -> Result<f64> {
        let start = get_time()?;
        self.cmd(Cmd::GetTime)?;
        Ok(get_time()? - start)
    }
}

pub struct Robot<T> {
    pub transport: T,
}

impl<T: RemoteRobotTransport> Robot<T> {
    pub fn new(transport: T) -> Self {
        Self { transport }
    }

    #[cfg(feature = "camloc")]
    pub fn get_position(&self) -> Result<Option<roblib::Position>> {
        roblib::cmd::parse_position_data(&self.transport.cmd(Cmd::GetPosition)?)
    }
}

#[cfg(feature = "roland")]
impl<T: RemoteRobotTransport> roblib::roland::Roland for Robot<T> {
    fn drive(&self, left: f64, right: f64) -> Result<DriveResult> {
        self.transport.cmd(Cmd::MoveRobot(left, right))?;
        #[cfg(feature = "camloc")]
        return Ok(None);
        #[cfg(not(feature = "camloc"))]
        return Ok(());
    }

    fn drive_by_angle(&self, angle: f64, speed: f64) -> Result<DriveResult> {
        self.transport.cmd(Cmd::MoveRobotByAngle(angle, speed))?;
        #[cfg(feature = "camloc")]
        return Ok(None);
        #[cfg(not(feature = "camloc"))]
        return Ok(());
    }

    fn led(&self, r: bool, g: bool, b: bool) -> Result<()> {
        self.transport.cmd(Cmd::Led(r, g, b))?;
        Ok(())
    }

    fn servo(&self, degree: f64) -> Result<()> {
        self.transport.cmd(Cmd::ServoAbsolute(degree))?;
        Ok(())
    }

    fn buzzer(&self, pw: f64) -> Result<()> {
        self.transport.cmd(Cmd::Buzzer(pw))?;
        Ok(())
    }

    fn track_sensor(&self) -> Result<[bool; 4]> {
        roblib::cmd::parse_track_sensor_data(&self.transport.cmd(Cmd::TrackSensor)?)
    }

    fn ultra_sensor(&self) -> Result<f64> {
        roblib::cmd::parse_ultra_sensor_data(&self.transport.cmd(Cmd::UltraSensor)?)
    }

    fn stop(&self) -> Result<()> {
        self.transport.cmd(Cmd::StopRobot)?;
        Ok(())
    }
}
