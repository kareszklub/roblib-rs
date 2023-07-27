pub use anyhow::Result;
use rppal::gpio;

use crate::get_servo_pwm_durations;

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

impl<'g> crate::gpio::SubscribablePin for InputPin<'g> {
    fn subscribe(
        &mut self,
        handler: impl FnMut(bool) -> Result<()> + Send + Sync + 'static,
    ) -> Result<()> {
        todo!()
    }
}

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
        let (period, pulse_width) = get_servo_pwm_durations(degree);
        self.pin.set_pwm(period, pulse_width)?;
        Ok(())
    }

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
