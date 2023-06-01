#[cfg(feature = "roland")]
mod constants;
#[cfg(feature = "roland")]
pub mod roland;

#[cfg(feature = "roland")]
pub use roland::Roland;

#[cfg(not(feature = "roland"))]
pub fn try_init() -> anyhow::Result<()> {
    Gpio::new()?;
    Ok(())
}

pub use anyhow::Result;
use rppal::gpio::{Gpio, OutputPin};
use std::{collections::HashMap, sync::Mutex, time::Duration};

lazy_static::lazy_static! {
    pub static ref PINS: Mutex<HashMap<u8, OutputPin>> = Mutex::new(HashMap::new());
}

pub fn set(pin: u8, value: bool) -> Result<()> {
    let mut l = PINS.lock().unwrap();
    if let std::collections::hash_map::Entry::Vacant(e) = l.entry(pin) {
        e.insert(Gpio::new()?.get(pin)?.into_output());
    }
    match value {
        true => l.get_mut(&pin).unwrap().set_high(),
        false => l.get_mut(&pin).unwrap().set_low(),
    };

    Ok(())
}

pub fn pwm(pin: u8, hz: f64, cycle: f64) -> Result<()> {
    let mut l = PINS.lock().unwrap();
    if let std::collections::hash_map::Entry::Vacant(e) = l.entry(pin) {
        e.insert(Gpio::new()?.get(pin)?.into_output());
    }
    let p = l.get_mut(&pin).unwrap();

    if cycle == 0.0 {
        p.clear_pwm()?;
    } else {
        p.set_pwm_frequency(hz, cycle)?;
    }

    Ok(())
}

pub fn servo(pin: u8, degree: f64) -> Result<()> {
    let mut l = PINS.lock().unwrap();
    if let std::collections::hash_map::Entry::Vacant(e) = l.entry(pin) {
        e.insert(Gpio::new()?.get(pin)?.into_output());
    }

    servo_on_pin(l.get_mut(&pin).unwrap(), degree)?;

    Ok(())
}

pub(crate) fn servo_on_pin(pin: &mut OutputPin, degree: f64) -> Result<()> {
    let degree = ((clamp(degree, -90., 90.) as i64 + 90) as u64 * 11) + 500;
    pin.set_pwm(Duration::from_millis(20), Duration::from_micros(degree))?; // 50Hz
    Ok(())
}

fn clamp<T: PartialOrd>(x: T, min: T, max: T) -> T {
    if x < min {
        min
    } else if x > max {
        max
    } else {
        x
    }
}
