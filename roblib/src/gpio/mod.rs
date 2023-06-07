use anyhow::Result;

#[cfg(feature = "gpio-backend")]
pub mod backend;

pub trait Gpio {
    fn read_pin(&self, pin: u8) -> Result<bool>;
    fn set_pin(&self, pin: u8, value: bool) -> Result<()>;
    fn pwm(&self, pin: u8, hz: f64, cycle: f64) -> Result<()>;
    fn servo(&self, pin: u8, degree: f64) -> Result<()>;
}
