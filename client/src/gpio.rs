use anyhow::Result;
use roblib::{cmd, gpio::event::GpioPin};

use crate::{
    transports::{Subscribable, Transport},
    Robot,
};

impl<T: Transport> roblib::gpio::Gpio for Robot<T> {
    fn read_pin(&self, pin: u8) -> Result<bool> {
        self.transport.cmd(cmd::ReadPin(pin))
    }

    fn write_pin(&self, pin: u8, value: bool) -> Result<()> {
        self.transport.cmd(cmd::WritePin(pin, value))
    }

    fn pwm(&self, pin: u8, hz: f64, cycle: f64) -> Result<()> {
        self.transport.cmd(cmd::Pwm(pin, hz, cycle))
    }

    fn servo(&self, pin: u8, degree: f64) -> Result<()> {
        self.transport.cmd(cmd::Servo(pin, degree))
    }

    fn pin_mode(&self, pin: u8, mode: roblib::gpio::Mode) -> Result<()> {
        self.transport.cmd(cmd::PinMode(pin, mode))
    }
}

pub struct Pin<'r, T: Transport> {
    robot: &'r Robot<T>,
    pin: u8,
}
pub struct InputPin<'r, T: Transport> {
    robot: &'r Robot<T>,
    pin: u8,
}
pub struct OutputPin<'r, T: Transport> {
    robot: &'r Robot<T>,
    pin: u8,
}

impl<'r, T: Transport + 'r> roblib::gpio::TypedGpio<'r> for Robot<T> {
    type O = OutputPin<'r, T>;
    type I = InputPin<'r, T>;
    type P = Pin<'r, T>;

    fn pin(&'r self, pin: u8) -> Result<Self::P> {
        Ok(Pin { pin, robot: self })
    }

    fn input_pin(&'r self, pin: u8) -> Result<Self::I> {
        self.transport
            .cmd(cmd::PinMode(pin, roblib::gpio::Mode::Input))?;
        Ok(InputPin { pin, robot: self })
    }

    fn output_pin(&'r self, pin: u8) -> Result<Self::O> {
        self.transport
            .cmd(cmd::PinMode(pin, roblib::gpio::Mode::Output))?;
        Ok(OutputPin { pin, robot: self })
    }
}

impl<'r, T: Transport> roblib::gpio::Pin for Pin<'r, T> {
    type I = InputPin<'r, T>;
    type O = OutputPin<'r, T>;

    fn get_pin(&self) -> u8 {
        self.pin
    }

    fn set_to_input(self) -> Result<Self::I> {
        let Self { pin, robot } = self;
        robot
            .transport
            .cmd(cmd::PinMode(pin, roblib::gpio::Mode::Input))?;
        Ok(InputPin { pin, robot })
    }

    fn set_to_output(self) -> Result<Self::O> {
        let Self { pin, robot } = self;
        robot
            .transport
            .cmd(cmd::PinMode(pin, roblib::gpio::Mode::Input))?;
        Ok(OutputPin { pin, robot })
    }
}

impl<'r, T: Transport> roblib::gpio::Pin for InputPin<'r, T> {
    type I = InputPin<'r, T>;
    type O = OutputPin<'r, T>;

    fn get_pin(&self) -> u8 {
        self.pin
    }

    fn set_to_input(self) -> Result<Self::I> {
        Ok(self)
    }

    fn set_to_output(self) -> Result<Self::O> {
        let Self { pin, robot } = self;
        robot
            .transport
            .cmd(cmd::PinMode(pin, roblib::gpio::Mode::Output))?;
        Ok(OutputPin { pin, robot })
    }
}
impl<'r, T: Transport> roblib::gpio::InputPin for InputPin<'r, T> {
    type O = OutputPin<'r, T>;
    type P = Pin<'r, T>;

    fn read(&self) -> Result<bool> {
        self.robot.transport.cmd(cmd::ReadPin(self.pin))
    }

    fn set_to_pin(self) -> Result<Self::P> {
        Ok(Pin {
            pin: self.pin,
            robot: self.robot,
        })
    }
}

impl<'r, T: Transport + Subscribable + Send + Sync> roblib::gpio::SubscribablePin
    for InputPin<'r, T>
{
    fn subscribe(
        &mut self,
        handler: impl FnMut(bool) -> Result<()> + Send + Sync + 'static,
    ) -> Result<()> {
        self.robot.transport.subscribe(GpioPin(self.pin), handler)
    }
}

impl<'r, T: Transport> roblib::gpio::Pin for OutputPin<'r, T> {
    type I = InputPin<'r, T>;
    type O = OutputPin<'r, T>;

    fn get_pin(&self) -> u8 {
        self.pin
    }

    fn set_to_input(self) -> Result<Self::I> {
        let Self { pin, robot } = self;
        robot
            .transport
            .cmd(cmd::PinMode(pin, roblib::gpio::Mode::Output))?;
        Ok(InputPin { pin, robot })
    }
    fn set_to_output(self) -> Result<Self::O> {
        let Self { pin, robot } = self;
        robot
            .transport
            .cmd(cmd::PinMode(pin, roblib::gpio::Mode::Output))?;
        Ok(OutputPin { pin, robot })
    }
}
impl<'r, T: Transport> roblib::gpio::OutputPin for OutputPin<'r, T> {
    type I = InputPin<'r, T>;
    type P = Pin<'r, T>;

    fn set(&mut self, value: bool) -> Result<()> {
        self.robot.transport.cmd(cmd::WritePin(self.pin, value))
    }

    fn pwm(&mut self, hz: f64, cycle: f64) -> Result<()> {
        self.robot.transport.cmd(cmd::Pwm(self.pin, hz, cycle))
    }

    fn servo(&mut self, degree: f64) -> Result<()> {
        self.robot.transport.cmd(cmd::Servo(self.pin, degree))
    }

    fn set_to_pin(self) -> Result<Self::P> {
        Ok(Pin {
            pin: self.pin,
            robot: self.robot,
        })
    }
}
