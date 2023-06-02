#[macro_use]
extern crate log;

use anyhow::Result;
use rppal::gpio::OutputPin;
use std::time::Duration;

pub mod cmd;

#[cfg(feature = "gpio")]
pub mod gpio;
#[cfg(feature = "roland")]
pub mod roland;
#[cfg(feature = "camloc")]
pub use camloc_server::{self, Position};

pub struct Robot {
    #[cfg(feature = "gpio")]
    pub raw_gpio: Option<gpio::Robot>,
    #[cfg(feature = "roland")]
    pub roland: Option<roland::GPIORoland>,
    #[cfg(feature = "camloc")]
    pub camloc_service: Option<camloc_server::service::LocationServiceHandle>,
}

impl Robot {
    #[cfg(feature = "camloc")]
    fn get_position(&self) -> Result<Option<camloc_server::Position>> {
        Ok(if let Some(s) = &self.camloc_service {
            // TODO: have camloc accept a runtime and block on that
            todo!()
            // s.get_position().position
        } else {
            None
        })
    }
}

#[cfg(any(feature = "roland", feature = "gpio"))]
pub(crate) fn servo_on_pin(pin: &mut OutputPin, degree: f64) -> Result<()> {
    let degree = ((clamp(degree, -90., 90.) as i64 + 90) as u64 * 11) + 500;
    pin.set_pwm(Duration::from_millis(20), Duration::from_micros(degree))?; // 50Hz
    Ok(())
}

#[allow(unused)]
fn clamp<T: PartialOrd>(x: T, min: T, max: T) -> T {
    if x < min {
        min
    } else if x > max {
        max
    } else {
        x
    }
}
