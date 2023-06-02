pub use anyhow::Result;
use rppal::gpio::{Gpio, OutputPin};
use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Mutex,
};

use crate::servo_on_pin;

pub struct Robot {
    pins: Mutex<HashMap<u8, OutputPin>>,
}

impl Robot {
    pub fn new() -> Result<Self> {
        // initalize everything, error on startup if something goes wrong
        drop(std::hint::black_box(Gpio::new()?));

        Ok(Self {
            pins: Mutex::new(HashMap::new()),
        })
    }

    pub fn modify_pin(&self, pin: u8, f: impl FnOnce(&mut OutputPin) -> Result<()>) -> Result<()> {
        let mut l = self.pins.lock().unwrap();

        match l.entry(pin) {
            Entry::Occupied(mut o) => f(o.get_mut())?,
            Entry::Vacant(v) => {
                let mut p = Gpio::new()?.get(pin)?.into_output();
                f(&mut p)?;
                v.insert(p);
            }
        }
        Ok(())
    }

    pub fn set(&self, pin: u8, value: bool) -> Result<()> {
        self.modify_pin(pin, |p| {
            if value {
                p.set_high();
            } else {
                p.set_low();
            }
            Ok(())
        })
    }

    pub fn pwm(&self, pin: u8, hz: f64, cycle: f64) -> Result<()> {
        self.modify_pin(pin, |p| {
            if cycle == 0. {
                p.clear_pwm()?;
            } else {
                p.set_pwm_frequency(hz, cycle)?;
            }

            Ok(())
        })
    }

    pub fn servo(&self, pin: u8, degree: f64) -> Result<()> {
        self.modify_pin(pin, |p| servo_on_pin(p, degree))
    }
}
