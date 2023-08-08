use std::time::{Duration, Instant};

pub use anyhow::Result;
use rppal::gpio;

use crate::map_num_range;

pub struct TypedGpioBackend {
    gpio: gpio::Gpio,
}
pub struct Pin<'g> {
    _gpio: &'g gpio::Gpio,
    pin: gpio::Pin,
}
pub struct InputPin<'g> {
    gpio: &'g gpio::Gpio,
    pin: gpio::InputPin,
}
pub struct OutputPin<'g> {
    gpio: &'g gpio::Gpio,
    pin: gpio::OutputPin,
}

impl TypedGpioBackend {
    pub fn new() -> Result<Self> {
        Ok(Self {
            gpio: gpio::Gpio::new()?,
        })
    }
}

impl<'g> crate::gpio::Pin for Pin<'g> {
    type I = InputPin<'g>;
    type O = OutputPin<'g>;

    fn get_pin(&self) -> u8 {
        self.pin.pin()
    }

    fn set_to_output(self) -> Result<Self::O> {
        todo!()
    }

    fn set_to_input(self) -> Result<Self::I> {
        todo!()
    }
}

impl<'g> crate::gpio::TypedGpio<'g> for TypedGpioBackend {
    type O = OutputPin<'g>;
    type I = InputPin<'g>;
    type P = Pin<'g>;

    fn pin(&'g self, pin: u8) -> Result<Self::P> {
        Ok(Pin {
            pin: self.gpio.get(pin)?,
            _gpio: &self.gpio,
        })
    }

    fn input_pin(&'g self, pin: u8) -> Result<Self::I> {
        Ok(InputPin {
            pin: self.gpio.get(pin)?.into_input(),
            gpio: &self.gpio,
        })
    }

    fn output_pin(&'g self, pin: u8) -> Result<Self::O> {
        Ok(OutputPin {
            pin: self.gpio.get(pin)?.into_output(),
            gpio: &self.gpio,
        })
    }
}

impl<'g> crate::gpio::Pin for InputPin<'g> {
    type I = InputPin<'g>;
    type O = OutputPin<'g>;

    fn get_pin(&self) -> u8 {
        self.pin.pin()
    }

    fn set_to_output(self) -> Result<<Self as crate::gpio::InputPin>::O> {
        let pin = self.pin.pin();
        let gpio = self.gpio;
        drop(self);

        Ok(OutputPin {
            gpio,
            pin: gpio.get(pin)?.into_output(),
        })
    }

    fn set_to_input(self) -> Result<Self::I> {
        Ok(self)
    }
}
impl<'g> crate::gpio::InputPin for InputPin<'g> {
    type O = OutputPin<'g>;
    type P = Pin<'g>;

    fn read(&self) -> Result<bool> {
        Ok(self.pin.read() == gpio::Level::High)
    }

    fn set_to_pin(self) -> Result<Self::P> {
        let pin = self.pin.pin();
        let gpio = self.gpio;
        drop(self);

        Ok(Pin {
            pin: gpio.get(pin)?,
            _gpio: gpio,
        })
    }
}

// TODO: impl typed gpio backend subscribe
// impl<'g> crate::gpio::SubscribablePin for InputPin<'g> {
//     fn subscribe(
//         &mut self,
//         _handler: impl FnMut(bool) -> Result<()> + Send + Sync + 'static,
//     ) -> Result<()> {
//         todo!()
//     }
// }

impl<'g> crate::gpio::Pin for OutputPin<'g> {
    type I = InputPin<'g>;
    type O = OutputPin<'g>;

    fn get_pin(&self) -> u8 {
        self.pin.pin()
    }

    fn set_to_output(self) -> Result<Self::O> {
        Ok(self)
    }

    fn set_to_input(self) -> Result<<Self as crate::gpio::OutputPin>::I> {
        let pin = self.pin.pin();
        let gpio = self.gpio;
        drop(self);

        Ok(InputPin {
            gpio,
            pin: gpio.get(pin)?.into_input(),
        })
    }
}
impl<'g> crate::gpio::OutputPin for OutputPin<'g> {
    type I = InputPin<'g>;
    type P = Pin<'g>;

    fn set(&mut self, value: bool) -> Result<()> {
        self.pin.write(value.into());
        Ok(())
    }

    fn pwm(&mut self, hz: f64, cycle: f64) -> Result<()> {
        if cycle == 0. {
            self.pin.clear_pwm()?;
        } else {
            self.pin.set_pwm_frequency(hz, cycle)?;
        }
        Ok(())
    }

    fn servo(&mut self, degree: f64) -> Result<()> {
        let degree = degree.clamp(-90., 90.);
        log::debug!("Enabling servo pwm");

        let now = Instant::now();
        self.pin.set_high();

        let dur = Duration::from_secs_f64(map_num_range(degree, -90., 90., 0.000750, 0.002250));

        std::thread::sleep(dur - Duration::from_micros(100));
        let until = now + dur;
        while Instant::now() > until {
            std::hint::spin_loop();
        }

        log::debug!("Disabling servo pwm");
        self.pin.set_low();

        Ok(())
    }
    // fn roland_servo(&self, degree: f64) -> Result<()> {
    //     let degree = degree.clamp(-90., 90.);
    //
    //     let mut lock = self.servo.lock().unwrap();
    //     lock.1 += 1;
    //     let id = lock.1;
    //     log::debug!("Enabling servo pwm: id: {id}");
    //
    //     let now = Instant::now();
    //     lock.0.set_high();
    //     drop(lock);
    //
    //     let dur = Duration::from_secs_f64(map_num_range(degree, -90., 90., 0.000750, 0.002250));
    //
    //     std::thread::sleep(dur - Duration::from_micros(100));
    //     let until = now + dur;
    //     while Instant::now() > until {
    //         std::hint::spin_loop();
    //     }
    //
    //     let mut lock = self.servo.lock().unwrap();
    //     log::debug!("Disabling servo pwm: id: {id} (latest: {})", lock.1);
    //     if lock.1 == id {
    //         lock.0.set_low();
    //     }
    //
    //     Ok(())
    // }

    fn set_to_pin(self) -> Result<Self::P> {
        let pin = self.pin.pin();
        let gpio = self.gpio;
        drop(self);

        Ok(Pin {
            _gpio: gpio,
            pin: gpio.get(pin)?,
        })
    }
}
