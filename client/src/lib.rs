#[macro_use]
extern crate log;
pub mod http;
pub mod logger;
pub mod ws;

pub use actix_rt::{main, task, time::sleep};
pub use anyhow::Result;
pub use camloc_common::Position;
use futures::executor::block_on;
use http::RobotHTTP;
pub use roblib::cmd;

use roblib::cmd::{get_time, Cmd};
use ws::RobotWS;

pub trait RemoteRobot {
    fn cmd(&self, cmd: Cmd) -> Result<String>;

    fn measure_latency(&self) -> Result<f64> {
        let start = get_time()?;
        self.cmd(Cmd::GetTime)?;
        Ok(get_time()? - start)
    }
}

macro_rules! impl_robo {
    ($R:tt) => {
        #[cfg(feature = "roland")]
        impl roblib::gpio::roland::Robot for $R {
            type Args = String;

            fn start(args: Self::Args) -> Result<Self> {
                block_on($R::connect(&args))
            }

            fn drive(&self, left: f64, right: f64) -> Result<()> {
                self.cmd(Cmd::MoveRobot(left, right))?;
                Ok(())
            }

            fn drive_by_angle(&self, angle: f64, speed: f64) -> Result<()> {
                self.cmd(Cmd::MoveRobotByAngle(angle, speed))?;
                Ok(())
            }

            fn led(&self, r: bool, g: bool, b: bool) -> Result<()> {
                self.cmd(Cmd::Led(r, g, b))?;
                Ok(())
            }

            fn servo(&self, degree: f64) -> Result<()> {
                self.cmd(Cmd::ServoAbsolute(degree))?;
                Ok(())
            }

            fn buzzer(&self, pw: f64) -> Result<()> {
                self.cmd(Cmd::Buzzer(pw))?;
                Ok(())
            }

            fn get_position(&self) -> Result<Option<Position>> {
                roblib::cmd::parse_position_data(&self.cmd(Cmd::GetPosition)?)
            }

            fn track_sensor(&self) -> Result<[bool; 4]> {
                roblib::cmd::parse_track_sensor_data(&self.cmd(Cmd::TrackSensor)?)
            }

            fn ultra_sensor(&self) -> Result<f64> {
                roblib::cmd::parse_ultra_sensor_data(&self.cmd(Cmd::UltraSensor)?)
            }

            fn stop(&self) -> Result<()> {
                self.cmd(Cmd::StopRobot)?;
                Ok(())
            }
        }
    };
}

impl_robo!(RobotWS);
impl_robo!(RobotHTTP);
