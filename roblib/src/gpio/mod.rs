#[cfg(feature = "roland")]
mod constants;
#[cfg(feature = "roland")]
pub mod roland;

#[cfg(feature = "roland")]
pub use roland::try_init;

#[cfg(not(feature = "roland"))]
pub fn try_init() -> anyhow::Result<()> {
    Gpio::new()?;
    Ok(())
}

pub use anyhow::Result;
use rppal::gpio::{Gpio, OutputPin};
use std::{collections::HashMap, sync::Mutex};

lazy_static::lazy_static! {
    pub static ref PINS: Mutex<HashMap<u8, OutputPin>> = Mutex::new(HashMap::new());
}

pub fn set(pin: u8, value: bool) -> Result<()> {
    let mut l = PINS.lock().unwrap();
    if !l.contains_key(&pin) {
        l.insert(pin, Gpio::new()?.get(pin)?.into_output());
    }
    match value {
        true => l.get_mut(&pin).unwrap().set_high(),
        false => l.get_mut(&pin).unwrap().set_low(),
    };

    Ok(())
}

pub fn pwm(pin: u8, hz: f64, cycle: f64) -> Result<()> {
    let mut l = PINS.lock().unwrap();
    if !l.contains_key(&pin) {
        l.insert(pin, Gpio::new()?.get(pin)?.into_output());
    }
    let p = l.get_mut(&pin).unwrap();

    if cycle == 0.0 {
        p.clear_pwm()?;
    } else {
        p.set_pwm_frequency(hz, cycle)?;
    }

    Ok(())
}
