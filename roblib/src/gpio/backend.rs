pub use anyhow::Result;
use rppal::gpio::{self, InputPin, OutputPin};
use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Mutex,
};

use crate::get_servo_pwm_durations;

struct MyPin {
    pwm_running: bool,
    callbacks: Vec<Box<dyn super::event::Subscriber + Send>>,
    pin: u8,
}
pub struct RobotPin<'a, 'b, 'c, P> {
    pwm_running: &'a mut bool,
    callbacks: &'b mut Vec<Box<dyn super::event::Subscriber + Send>>,
    pin: &'c mut P,
}

pub struct GpioBackend {
    pins: Mutex<HashMap<u8, MyPin>>,
    gpio: gpio::Gpio,
}

pub trait PinMode: Into<gpio::Mode> {
    type P;
    fn to(pin: gpio::Pin) -> Self::P;
}
pub struct InputMode;
impl From<InputMode> for gpio::Mode {
    fn from(_: InputMode) -> Self {
        gpio::Mode::Input
    }
}
impl PinMode for InputMode {
    type P = InputPin;
    fn to(pin: gpio::Pin) -> Self::P {
        pin.into_input()
    }
}
pub struct OutputMode;
impl From<OutputMode> for gpio::Mode {
    fn from(_: OutputMode) -> Self {
        gpio::Mode::Output
    }
}
impl PinMode for OutputMode {
    type P = OutputPin;
    fn to(pin: gpio::Pin) -> Self::P {
        pin.into_output()
    }
}

impl GpioBackend {
    pub fn new() -> Result<Self> {
        Ok(Self {
            gpio: gpio::Gpio::new()?,
            pins: Mutex::new(HashMap::new()),
        })
    }

    pub fn subscribe(
        &self,
        pin: u8,
        callback: Box<dyn super::event::Subscriber + Send>,
    ) -> Result<()> {
        self.use_pin::<InputMode, ()>(pin, |p| p.callbacks.push(callback))
    }
    pub fn modify_subscriptions<T>(
        &self,
        pin: u8,
        f: impl FnOnce(&mut Vec<Box<dyn super::event::Subscriber + Send>>) -> T,
    ) -> Result<T> {
        self.use_pin::<InputMode, T>(pin, |p| f(p.callbacks))
    }

    pub fn use_pin<M: PinMode, T>(
        &self,
        pin: u8,
        f: impl FnOnce(&mut RobotPin<M::P>) -> T,
    ) -> Result<T> {
        let mut l = self.pins.lock().unwrap();

        match l.entry(pin) {
            Entry::Occupied(mut o) => {
                let mp = o.get_mut();
                let p = self.gpio.get(mp.pin)?;
                let mut p = M::to(p);

                Ok(f(&mut RobotPin {
                    pwm_running: &mut mp.pwm_running,
                    callbacks: &mut mp.callbacks,
                    pin: &mut p,
                }))
            }

            Entry::Vacant(v) => {
                let mut mp = MyPin {
                    pwm_running: false,
                    callbacks: vec![],
                    pin,
                };

                let p = self.gpio.get(mp.pin)?;
                let mut p = M::to(p);

                let ret = f(&mut RobotPin {
                    pwm_running: &mut mp.pwm_running,
                    callbacks: &mut mp.callbacks,
                    pin: &mut p,
                });

                v.insert(mp);

                Ok(ret)
            }
        }
    }
}

impl super::Gpio for GpioBackend {
    fn set_pin(&self, pin: u8, value: bool) -> Result<()> {
        self.use_pin::<OutputMode, ()>(pin, |p| p.pin.write(value.into()))
    }

    fn pwm(&self, pin: u8, hz: f64, cycle: f64) -> Result<()> {
        self.use_pin::<OutputMode, Result<()>>(pin, |p| {
            if cycle == 0. {
                p.pin.clear_pwm()?;
                *p.pwm_running = false;
            } else {
                p.pin.set_pwm_frequency(hz, cycle)?;
                *p.pwm_running = true;
            }
            Ok(())
        })?
    }

    fn servo(&self, pin: u8, degree: f64) -> Result<()> {
        let (period, pulse_width) = get_servo_pwm_durations(degree);
        self.use_pin::<OutputMode, Result<()>>(pin, |p| Ok(p.pin.set_pwm(period, pulse_width)?))?
    }

    fn read_pin(&self, pin: u8) -> Result<bool> {
        self.use_pin::<InputMode, bool>(pin, |p| p.pin.is_high())
    }
}
