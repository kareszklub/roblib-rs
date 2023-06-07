pub use anyhow::Result;
use rppal::gpio::{self, IoPin, Mode};
use std::{
    collections::{hash_map::Entry, HashMap},
    ops::{Deref, DerefMut},
    sync::Mutex,
};

use crate::get_servo_pwm_durations;

pub struct RobotPin {
    pwm_running: bool,
    pin: IoPin,
}

impl Deref for RobotPin {
    type Target = IoPin;

    fn deref(&self) -> &Self::Target {
        &self.pin
    }
}
impl DerefMut for RobotPin {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.pin
    }
}

pub struct GpioBackend {
    pins: Mutex<HashMap<u8, RobotPin>>,
}

impl GpioBackend {
    pub fn new() -> Result<Self> {
        // initalize everything, error on startup if something goes wrong
        drop(std::hint::black_box(gpio::Gpio::new()?));

        Ok(Self {
            pins: Mutex::new(HashMap::new()),
        })
    }

    pub fn use_pin<T>(
        &self,
        pin: u8,
        mode: Mode,
        f: impl FnOnce(&mut RobotPin) -> Result<T>,
    ) -> Result<T> {
        let mut l = self.pins.lock().unwrap();

        match l.entry(pin) {
            Entry::Occupied(mut o) => {
                let p = o.get_mut();
                if p.pwm_running {
                    p.clear_pwm()?;
                    p.pwm_running = false;
                }

                p.set_mode(mode);
                f(p)
            }

            Entry::Vacant(v) => {
                let mut p = RobotPin {
                    pin: gpio::Gpio::new()?.get(pin)?.into_io(mode),
                    pwm_running: false,
                };

                let ret = f(&mut p);

                v.insert(p);

                ret
            }
        }
    }
}

impl super::Gpio for GpioBackend {
    fn set_pin(&self, pin: u8, value: bool) -> Result<()> {
        self.use_pin(pin, Mode::Output, |p| {
            if value {
                p.set_high();
            } else {
                p.set_low();
            }
            Ok(())
        })
    }

    fn pwm(&self, pin: u8, hz: f64, cycle: f64) -> Result<()> {
        self.use_pin(pin, Mode::Output, |p| {
            if cycle == 0. {
                p.clear_pwm()?;
                p.pwm_running = false;
            } else {
                p.set_pwm_frequency(hz, cycle)?;
                p.pwm_running = true;
            }
            Ok(())
        })
    }

    fn servo(&self, pin: u8, degree: f64) -> Result<()> {
        let (period, pulse_width) = get_servo_pwm_durations(degree);
        Ok(self.use_pin(pin, Mode::Output, |p| Ok(p.set_pwm(period, pulse_width)))??)
    }

    fn read_pin(&self, pin: u8) -> Result<bool> {
        self.use_pin(pin, Mode::Input, |p| Ok(p.is_high()))
    }
}
