pub mod logger;

pub mod transports;

use std::time::{Duration, Instant};

pub use anyhow::Result;

pub use roblib;

use roblib::{cmd, event::Event, RoblibRobot};
use transports::{Subscribable, Transport};

pub struct Robot<T> {
    transport: T,
}

impl<T: Transport> Robot<T> {
    pub fn new(transport: T) -> Self {
        Self { transport }
    }

    pub fn measure_latency(&self) -> Result<Duration> {
        let start = Instant::now();
        let _ = self.transport.cmd(cmd::GetUptime)?;
        Ok(Instant::now() - start)
    }
    pub fn get_server_uptime(&self) -> Result<Duration> {
        self.transport.cmd(cmd::GetUptime)
    }
}
impl<T: Subscribable> Robot<T> {
    pub fn subscribe<E: Event>(
        &self,
        ev: E,
        handler: impl FnMut(E::Item) -> Result<()> + Send + Sync + 'static,
    ) -> Result<()> {
        self.transport.subscribe(ev, handler)
    }
    pub fn unsubscribe<E: Event>(&self, ev: E) -> Result<()> {
        self.transport.unsubscribe(ev)
    }
}

// #[cfg(feature = "async")]
// pub struct RobotAsync<T> {
//     pub transport: T,
// }

// #[cfg(feature = "async")]
// impl<T: transports::TransportAsync> RobotAsync<T> {
//     pub fn new(transport: T) -> Self {
//         Self { transport }
//     }

//     pub async fn measure_latency(&self) -> Result<Duration> {
//         let start = Instant::now();
//         self.transport.cmd(cmd::GetUptime).await?;
//         Ok(Instant::now() - start)
//     }
//     pub async fn get_server_uptime(&self) -> Result<Duration> {
//         self.transport.cmd(cmd::GetUptime).await
//     }
// }
// #[cfg(feature = "async")]
// impl<T: transports::SubscribableAsync> RobotAsync<T> {
//     pub async fn subscribe<E, F, R>(&self, ev: E, handler: F) -> Result<()>
//     where
//         E: Event + Send,
//         E::Item: Send + Sync,
//         F: (FnMut(E::Item) -> R) + Send + Sync + 'static,
//         R: std::future::Future<Output = Result<()>> + Send + Sync,
//     {
//         self.transport.subscribe(ev, handler).await
//     }
//     pub async fn unsubscribe<E>(&self, ev: E) -> Result<()>
//     where
//         E: Event + Send,
//     {
//         self.transport.unsubscribe(ev).await
//     }
// }

impl<T: Transport> RoblibRobot for Robot<T> {
    fn nop(&self) -> anyhow::Result<()> {
        self.transport.cmd(cmd::Nop)
    }

    fn get_uptime(&self) -> anyhow::Result<Duration> {
        self.transport.cmd(cmd::GetUptime)
    }
}

#[cfg(feature = "roland")]
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
impl<T: Transport> roblib::gpio::Gpio for Robot<T> {
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
impl<T: Transport> roblib::camloc::Camloc for Robot<T> {
    fn get_position(&self) -> Result<Option<roblib::camloc::Position>> {
        self.transport.cmd(cmd::GetPosition)
    }
}

// #[cfg(feature = "async")]
// #[cfg_attr(feature = "async", async_trait::async_trait)]
// impl<T: transports::TransportAsync> roblib::RoblibRobotAsync for RobotAsync<T> {
//     async fn nop(&self) -> Result<()> {
//         self.transport.cmd(cmd::Nop).await
//     }

//     async fn get_uptime(&self) -> Result<Duration> {
//         self.transport.cmd(cmd::GetUptime).await
//     }
// }

// #[cfg(all(feature = "roland", feature = "async"))]
// #[cfg_attr(feature = "async", async_trait::async_trait)]
// impl<T: transports::TransportAsync> roblib::roland::RolandAsync for RobotAsync<T> {
//     async fn drive(&self, left: f64, right: f64) -> Result<()> {
//         if !(-1. ..=1.).contains(&left) || !(-1. ..=1.).contains(&right) {
//             warn!("Drive values are now [-1, 1] not [-100, 100]");
//         }
//         let r = self.transport.cmd(cmd::MoveRobot(left, right)).await?;
//         Ok(r)
//     }

//     async fn drive_by_angle(&self, angle: f64, speed: f64) -> Result<()> {
//         if !(-1. ..=1.).contains(&speed) {
//             warn!("Drive values are now [-1, 1] not [-100, 100]");
//         }
//         self.transport
//             .cmd(cmd::MoveRobotByAngle(angle, speed))
//             .await
//     }

//     async fn led(&self, r: bool, g: bool, b: bool) -> Result<()> {
//         self.transport.cmd(cmd::Led(r, g, b)).await
//     }

//     async fn servo(&self, degree: f64) -> Result<()> {
//         self.transport.cmd(cmd::ServoAbsolute(degree)).await
//     }

//     async fn buzzer(&self, pw: f64) -> Result<()> {
//         self.transport.cmd(cmd::Buzzer(pw)).await
//     }

//     async fn track_sensor(&self) -> Result<[bool; 4]> {
//         self.transport.cmd(cmd::TrackSensor).await
//     }

//     async fn ultra_sensor(&self) -> Result<f64> {
//         self.transport.cmd(cmd::UltraSensor).await
//     }

//     async fn stop(&self) -> Result<()> {
//         self.transport.cmd(cmd::StopRobot).await
//     }
// }

// #[cfg(all(feature = "gpio", feature = "async"))]
// #[cfg_attr(feature = "async", async_trait::async_trait)]
// impl<T: transports::TransportAsync> roblib::gpio::GpioAsync for RobotAsync<T> {
//     async fn read_pin(&self, pin: u8) -> Result<bool> {
//         self.transport.cmd(cmd::ReadPin(pin)).await
//     }

//     async fn set_pin(&self, pin: u8, value: bool) -> Result<()> {
//         self.transport.cmd(cmd::SetPin(pin, value)).await
//     }

//     async fn pwm(&self, pin: u8, hz: f64, cycle: f64) -> Result<()> {
//         self.transport.cmd(cmd::SetPwm(pin, hz, cycle)).await
//     }

//     async fn servo(&self, pin: u8, degree: f64) -> Result<()> {
//         self.transport.cmd(cmd::ServoBasic(pin, degree)).await
//     }
// }

// #[cfg(all(feature = "camloc", feature = "async"))]
// #[cfg_attr(feature = "async", async_trait::async_trait)]
// impl<T: transports::TransportAsync> roblib::camloc::CamlocAsync for RobotAsync<T> {
//     async fn get_position(&self) -> Result<Option<roblib::camloc::Position>> {
//         self.transport.cmd(cmd::GetPosition).await
//     }
// }
