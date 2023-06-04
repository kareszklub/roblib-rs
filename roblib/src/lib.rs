#[macro_use]
extern crate log;

use std::time::{Duration, Instant};

pub mod cmd;

#[cfg(feature = "camloc")]
pub mod camloc {
    pub mod server {
        pub use camloc_server::*;
    }
    pub use camloc_server::{service::Subscriber, MotionHint, Mutex, Position, MAIN_PORT};
}

#[cfg(feature = "gpio")]
pub mod gpio;

#[cfg(feature = "roland")]
pub mod roland;

pub struct Robot {
    pub startup_time: Instant,

    #[cfg(feature = "gpio")]
    pub raw_gpio: Option<gpio::Robot>,
    #[cfg(feature = "roland")]
    pub roland: Option<roland::GPIORoland>,
    #[cfg(feature = "camloc")]
    pub camloc_service: Option<camloc_server::service::LocationServiceHandle>,
}

#[cfg(any(feature = "roland", feature = "gpio"))]
pub(crate) fn get_servo_pwm_durations(degree: f64) -> (Duration, Duration) {
    let degree = ((degree.clamp(-90., 90.) as i64 + 90) as u64 * 11) + 500;
    (Duration::from_millis(20), Duration::from_micros(degree)) // 50Hz
}
