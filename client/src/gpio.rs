use anyhow::Result;
use roblib::cmd;

use crate::{transports::Transport, Robot};

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

impl<'r, T: Transport + 'r, S: roblib::gpio::Subscriber> roblib::gpio::Gpio<'r, S> for Robot<T> {
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

impl<'r, T: Transport, S: roblib::gpio::Subscriber> roblib::gpio::Pin<S> for Pin<'r, T> {
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

impl<'r, T: Transport, S: roblib::gpio::Subscriber> roblib::gpio::Pin<S> for InputPin<'r, T> {
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
            .cmd(cmd::PinMode(pin, roblib::gpio::Mode::Output))?;
        Ok(OutputPin { pin, robot })
    }
}
impl<'r, T: Transport, S: roblib::gpio::Subscriber> roblib::gpio::InputPin<S> for InputPin<'r, T> {
    type O = OutputPin<'r, T>;
    type P = Pin<'r, T>;

    fn read(&self) -> Result<bool> {
        self.robot.transport.cmd(cmd::ReadPin(self.pin))
    }

    fn subscribe(&self, sub: S) -> Result<()> {
        todo!()
    }

    fn set_to_output(self) -> Result<<Self as roblib::gpio::InputPin<S>>::O> {
        let Self { pin, robot } = self;
        robot
            .transport
            .cmd(cmd::PinMode(pin, roblib::gpio::Mode::Output))?;
        Ok(OutputPin { pin, robot })
    }
    fn set_to_pin(self) -> Result<Self::P> {
        Ok(Pin {
            pin: self.pin,
            robot: self.robot,
        })
    }
}

impl<'r, T: Transport, S: roblib::gpio::Subscriber> roblib::gpio::Pin<S> for OutputPin<'r, T> {
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
impl<'r, T: Transport, S: roblib::gpio::Subscriber> roblib::gpio::OutputPin<S>
    for OutputPin<'r, T>
{
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

    fn set_to_input(self) -> Result<<Self as roblib::gpio::OutputPin<S>>::I> {
        let Self { pin, robot } = self;
        robot
            .transport
            .cmd(cmd::PinMode(pin, roblib::gpio::Mode::Input))?;
        Ok(InputPin { pin, robot })
    }
    fn set_to_pin(self) -> Result<Self::P> {
        Ok(Pin {
            pin: self.pin,
            robot: self.robot,
        })
    }
}
