use crate::{
    get_servo_pwm_durations,
    gpio::{event::Event, Mode},
};
use rppal::gpio::{InputPin, OutputPin, Trigger};
use std::{
    collections::{hash_map::Entry, HashMap},
    sync::{
        atomic::{AtomicBool, Ordering::SeqCst},
        Arc, RwLock,
    },
};

pub trait Subscriber: Send + Sync {
    fn handle(&self, event: Event);
}

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

struct SubHandler {
    handler: Box<dyn Subscriber>,
    last_value: AtomicBool,
}

pub struct SimpleGpioBackend {
    pins: RwLock<HashMap<u8, Pin>>,
    handlers: Arc<RwLock<HashMap<u8, SubHandler>>>,
    gpio: rppal::gpio::Gpio,
}

impl SimpleGpioBackend {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            gpio: rppal::gpio::Gpio::new()?,
            pins: RwLock::new(HashMap::new()),
            handlers: Arc::new(RwLock::new(HashMap::new())),
        })
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

    fn input_pin_mut<F, R>(&self, pin: u8, f: F) -> anyhow::Result<R>
    where
        F: FnOnce(&mut InputPin) -> R,
    {
        match self.pins.write().unwrap().get_mut(&pin) {
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

    pub fn subscribe(&self, pin: u8, handler: Box<dyn Subscriber>) -> anyhow::Result<()> {
        self.input_pin_mut(pin, |p| {
            match self.handlers.write().unwrap().entry(pin) {
                Entry::Occupied(_) => {
                    Err(anyhow::anyhow!("Event handler already set on pin {pin}!!"))?
                }
                // Entry::Occupied(mut v) => v.get_mut().push(handler),
                Entry::Vacant(v) => {
                    let state = p.read() as u8 != 0;
                    v.insert(SubHandler {
                        handler,
                        last_value: state.into(),
                    });

                    let handlers = self.handlers.clone();
                    p.set_async_interrupt(Trigger::Both, move |l| {
                        let lock = handlers.read().unwrap();
                        let Some(handle) = lock.get(&pin) else {
                            return log::error!("Handlers removed without clearing interrupt!");
                        };
                        // rising/falling edge thing seems to be backwards
                        let value = l as u8 == 0;
                        let res = &handle.last_value.fetch_update(SeqCst, SeqCst, |last| {
                            log::debug!("({last}) {value}");
                            if value != last {
                                return Some(value);
                            }
                            None
                        });
                        if res.is_ok() {
                            let ev = Event::PinChanged(pin, value);
                            handle.handler.handle(ev)
                        }
                    })?;
                }
            }
            Ok(())
        })?
    }

    // fn fire_event(&self, pin: u8, value: bool) {
    //     let handlers = self.handlers.read().unwrap();
    //     if let Some(p) = handlers.get(&pin) {
    //         p.handle(Event::PinChanged(pin, value));
    //     }
    // }

    pub fn unsubscribe(&self, pin: u8) -> anyhow::Result<()> {
        self.input_pin_mut(pin, |p| p.clear_async_interrupt())??;
        let mut handlers = self.handlers.write().unwrap();
        handlers.remove(&pin);
        Ok(())
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
        let mut pins = self.pins.write().unwrap();
        match pins.entry(pin) {
            Entry::Occupied(v) => {
                if mode == v.get().mode() {
                    log::warn!("Pin {pin} mode already set to {mode:?}");
                } else {
                    log::warn!("Pin {pin} mode switched to {mode:?}");
                    // TODO: revisit
                    drop(v.remove());
                    let p = match mode {
                        Mode::Input => Pin::Input(self.gpio.get(pin)?.into_input()),
                        Mode::Output => Pin::Output(self.gpio.get(pin)?.into_output()),
                    };
                    pins.insert(pin, p);
                }
            }
            Entry::Vacant(v) => {
                let p = match mode {
                    Mode::Input => Pin::Input(self.gpio.get(pin)?.into_input()),
                    Mode::Output => Pin::Output(self.gpio.get(pin)?.into_output()),
                };
                v.insert(p);
            }
        }
        Ok(())
    }
}
