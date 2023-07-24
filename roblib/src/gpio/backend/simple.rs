use rppal::gpio::{InputPin, OutputPin};
use std::{
    collections::{hash_map::Entry, HashMap},
    sync::RwLock,
};

use crate::{get_servo_pwm_durations, gpio::Mode};

enum Pin {
    Input(InputPin),
    Output(OutputPin),
}
impl Pin {
    fn mode(&self) -> Mode {
        match self {
            Pin::Input(_) => Mode::Input,
            Pin::Output(_) => Mode::Output,
        }
    }
}

pub struct SimpleGpioBackend {
    pins: RwLock<HashMap<u8, Pin>>,
    gpio: rppal::gpio::Gpio,
}

impl SimpleGpioBackend {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            gpio: rppal::gpio::Gpio::new()?,
            pins: RwLock::new(HashMap::new()),
        })
    }

    fn insert_pin(&self, pin: u8, mode: Mode) -> anyhow::Result<()> {
        let p = match mode {
            Mode::Input => Pin::Input(self.gpio.get(pin)?.into_input()),
            Mode::Output => Pin::Output(self.gpio.get(pin)?.into_output()),
        };
        self.pins.write().unwrap().insert(pin, p);
        Ok(())
    }

    fn input_pin<F, R>(&self, pin: u8, f: F) -> anyhow::Result<R>
    where
        F: FnOnce(&InputPin) -> R,
    {
        match self.pins.read().unwrap().get(&pin) {
            Some(Pin::Input(p)) => Ok(f(p)),
            _ => Err(anyhow::anyhow!("Pin {pin} not set up for reading")),
        }
    }

    fn output_pin<F, R>(&self, pin: u8, f: F) -> anyhow::Result<R>
    where
        F: FnOnce(&mut OutputPin) -> R,
    {
        match self.pins.write().unwrap().get_mut(&pin) {
            Some(Pin::Output(p)) => Ok(f(p)),
            _ => Err(anyhow::anyhow!("Pin {pin} not set up writing")),
        }
    }
}

impl super::super::Gpio for SimpleGpioBackend {
    fn read_pin(&self, pin: u8) -> anyhow::Result<bool> {
        self.input_pin(pin, |p| p.is_high())
    }

    fn write_pin(&self, pin: u8, value: bool) -> anyhow::Result<()> {
        self.output_pin(pin, |p| p.write(value.into()))
    }

    fn pwm(&self, pin: u8, hz: f64, cycle: f64) -> anyhow::Result<()> {
        self.output_pin(pin, |p| {
            if cycle == 0. {
                p.clear_pwm()?;
            } else {
                p.set_pwm_frequency(hz, cycle)?;
            }
            Ok(())
        })?
    }

    fn servo(&self, pin: u8, degree: f64) -> anyhow::Result<()> {
        let (period, pulse_width) = get_servo_pwm_durations(degree);
        self.output_pin(pin, |p| Ok(p.set_pwm(period, pulse_width)?))?
    }

    fn pin_mode(&self, pin: u8, mode: Mode) -> anyhow::Result<()> {
        match self.pins.write().unwrap().entry(pin) {
            Entry::Occupied(v) => {
                if mode == v.get().mode() {
                    log::warn!("Pin {pin} mode already set to {mode:?}");
                } else {
                    log::warn!("Pin {pin} mode switched to {mode:?}");
                    // TODO: could maybe deadlock?? idk will have to check
                    self.insert_pin(pin, mode)?;
                }
            }
            Entry::Vacant(_) => self.insert_pin(pin, mode)?,
        }
        Ok(())
    }
}
