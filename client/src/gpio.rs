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

#[cfg(feature = "async")]
pub mod gpio_async {
    use crate::{
        async_robot::RobotAsync,
        transports::{SubscribableAsync, TransportAsync},
    };
    use anyhow::Result;
    use async_trait::async_trait;
    use roblib::{cmd, gpio::event::GpioPin};

    #[async_trait]
    impl<T: TransportAsync> roblib::gpio::GpioAsync for RobotAsync<T> {
        async fn read_pin(&self, pin: u8) -> Result<bool> {
            self.transport.cmd(cmd::ReadPin(pin)).await
        }

        async fn write_pin(&self, pin: u8, value: bool) -> Result<()> {
            self.transport.cmd(cmd::WritePin(pin, value)).await
        }

        async fn pwm(&self, pin: u8, hz: f64, cycle: f64) -> Result<()> {
            self.transport.cmd(cmd::Pwm(pin, hz, cycle)).await
        }

        async fn servo(&self, pin: u8, degree: f64) -> Result<()> {
            self.transport.cmd(cmd::Servo(pin, degree)).await
        }

        async fn pin_mode(&self, pin: u8, mode: roblib::gpio::Mode) -> Result<()> {
            self.transport.cmd(cmd::PinMode(pin, mode)).await
        }
    }

    pub struct PinAsync<'r, T: TransportAsync> {
        robot: &'r RobotAsync<T>,
        pin: u8,
    }
    pub struct InputPinAsync<'r, T: TransportAsync> {
        robot: &'r RobotAsync<T>,
        pin: u8,
    }
    pub struct OutputPinAsync<'r, T: TransportAsync> {
        robot: &'r RobotAsync<T>,
        pin: u8,
    }

    #[async_trait]
    impl<'r, T: TransportAsync + 'r> roblib::gpio::TypedGpioAsync<'r> for RobotAsync<T> {
        type O = OutputPinAsync<'r, T>;
        type I = InputPinAsync<'r, T>;
        type P = PinAsync<'r, T>;

        async fn pin(&'r self, pin: u8) -> Result<Self::P> {
            Ok(PinAsync { pin, robot: self })
        }

        async fn input_pin(&'r self, pin: u8) -> Result<Self::I> {
            self.transport
                .cmd(cmd::PinMode(pin, roblib::gpio::Mode::Input))
                .await?;
            Ok(InputPinAsync { pin, robot: self })
        }

        async fn output_pin(&'r self, pin: u8) -> Result<Self::O> {
            self.transport
                .cmd(cmd::PinMode(pin, roblib::gpio::Mode::Output))
                .await?;
            Ok(OutputPinAsync { pin, robot: self })
        }
    }

    #[async_trait]
    impl<'r, T: TransportAsync> roblib::gpio::PinAsync for PinAsync<'r, T> {
        type I = InputPinAsync<'r, T>;
        type O = OutputPinAsync<'r, T>;

        async fn get_pin(&self) -> u8 {
            self.pin
        }

        async fn set_to_input(self) -> Result<Self::I> {
            let Self { pin, robot } = self;
            robot
                .transport
                .cmd(cmd::PinMode(pin, roblib::gpio::Mode::Input))
                .await?;
            Ok(InputPinAsync { pin, robot })
        }

        async fn set_to_output(self) -> Result<Self::O> {
            let Self { pin, robot } = self;
            robot
                .transport
                .cmd(cmd::PinMode(pin, roblib::gpio::Mode::Input))
                .await?;
            Ok(OutputPinAsync { pin, robot })
        }
    }

    #[async_trait]
    impl<'r, T: TransportAsync> roblib::gpio::PinAsync for InputPinAsync<'r, T> {
        type I = InputPinAsync<'r, T>;
        type O = OutputPinAsync<'r, T>;

        async fn get_pin(&self) -> u8 {
            self.pin
        }

        async fn set_to_input(self) -> Result<Self::I> {
            Ok(self)
        }

        async fn set_to_output(self) -> Result<Self::O> {
            let Self { pin, robot } = self;
            robot
                .transport
                .cmd(cmd::PinMode(pin, roblib::gpio::Mode::Output))
                .await?;
            Ok(OutputPinAsync { pin, robot })
        }
    }
    #[async_trait]
    impl<'r, T: TransportAsync> roblib::gpio::InputPinAsync for InputPinAsync<'r, T> {
        type O = OutputPinAsync<'r, T>;
        type P = PinAsync<'r, T>;

        async fn read(&self) -> Result<bool> {
            self.robot.transport.cmd(cmd::ReadPin(self.pin)).await
        }

        async fn set_to_pin(self) -> Result<Self::P> {
            Ok(PinAsync {
                pin: self.pin,
                robot: self.robot,
            })
        }
    }

    #[async_trait]
    impl<'r, T: TransportAsync + SubscribableAsync + Send + Sync> roblib::gpio::SubscribablePinAsync
        for InputPinAsync<'r, T>
    {
        async fn subscribe<F, R>(&mut self, handler: F) -> Result<()>
        where
            F: (FnMut(bool) -> R) + Send + Sync + 'static,
            R: std::future::Future<Output = Result<()>> + Send + Sync,
        {
            self.robot
                .transport
                .subscribe(GpioPin(self.pin), handler)
                .await
        }
    }

    #[async_trait]
    impl<'r, T: TransportAsync> roblib::gpio::PinAsync for OutputPinAsync<'r, T> {
        type I = InputPinAsync<'r, T>;
        type O = OutputPinAsync<'r, T>;

        async fn get_pin(&self) -> u8 {
            self.pin
        }

        async fn set_to_input(self) -> Result<Self::I> {
            let Self { pin, robot } = self;
            robot
                .transport
                .cmd(cmd::PinMode(pin, roblib::gpio::Mode::Output))
                .await?;
            Ok(InputPinAsync { pin, robot })
        }
        async fn set_to_output(self) -> Result<Self::O> {
            let Self { pin, robot } = self;
            robot
                .transport
                .cmd(cmd::PinMode(pin, roblib::gpio::Mode::Output))
                .await?;
            Ok(OutputPinAsync { pin, robot })
        }
    }
    #[async_trait]
    impl<'r, T: TransportAsync> roblib::gpio::OutputPinAsync for OutputPinAsync<'r, T> {
        type I = InputPinAsync<'r, T>;
        type P = PinAsync<'r, T>;

        async fn set(&mut self, value: bool) -> Result<()> {
            self.robot
                .transport
                .cmd(cmd::WritePin(self.pin, value))
                .await
        }

        async fn pwm(&mut self, hz: f64, cycle: f64) -> Result<()> {
            self.robot
                .transport
                .cmd(cmd::Pwm(self.pin, hz, cycle))
                .await
        }

        async fn servo(&mut self, degree: f64) -> Result<()> {
            self.robot.transport.cmd(cmd::Servo(self.pin, degree)).await
        }

        async fn set_to_pin(self) -> Result<Self::P> {
            Ok(PinAsync {
                pin: self.pin,
                robot: self.robot,
            })
        }
    }
}
