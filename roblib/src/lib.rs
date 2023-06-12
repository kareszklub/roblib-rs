extern crate log;

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

#[cfg(feature = "gpio-backend")]
pub(crate) fn get_servo_pwm_durations(degree: f64) -> (std::time::Duration, std::time::Duration) {
    let degree = ((degree.clamp(-90., 90.) as i64 + 90) as u64 * 11) + 500;
    (
        std::time::Duration::from_millis(20),
        std::time::Duration::from_micros(degree),
    ) // 50Hz
}
