#![allow(unused_imports, unused_variables)]
use roblib::{
    cmd::{PinMode, Pwm, ReadPin, Servo, WritePin},
    gpio::Gpio,
};
use std::sync::Arc;

use super::{Backends, Execute};

#[async_trait::async_trait]
impl Execute for PinMode {
    async fn execute(&self, robot: Arc<Backends>) -> anyhow::Result<Self::Return> {
        let PinMode(pin, mode) = *self;

        debug!("Pinmode: {pin} {mode:?}");

        #[cfg(feature = "backend")]
        if let Some(r) = &robot.raw_gpio {
            r.pin_mode(pin, mode)?
        };

        Ok(())
    }
}

#[async_trait::async_trait]
impl Execute for ReadPin {
    async fn execute(&self, robot: Arc<Backends>) -> anyhow::Result<Self::Return> {
        let ReadPin(pin) = *self;

        debug!("Read pin: {pin}");

        #[cfg(feature = "backend")]
        let res = if let Some(r) = &robot.raw_gpio {
            r.read_pin(pin)?
        } else {
            false
        };

        #[cfg(not(feature = "backend"))]
        let res = false;

        Ok(res)
    }
}

#[async_trait::async_trait]
impl Execute for WritePin {
    async fn execute(&self, robot: Arc<Backends>) -> anyhow::Result<Self::Return> {
        let WritePin(pin, value) = *self;

        debug!("Set pin: {pin}:{value}");

        #[cfg(feature = "backend")]
        if let Some(r) = &robot.raw_gpio {
            r.set_pin(pin, value)?;
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl Execute for Pwm {
    async fn execute(&self, robot: Arc<Backends>) -> anyhow::Result<Self::Return> {
        let Pwm(pin, hz, cycle) = *self;

        debug!("Set pwm: {pin}:{hz}:{cycle}");

        #[cfg(feature = "backend")]
        if let Some(r) = &robot.raw_gpio {
            r.pwm(pin, hz, cycle)?;
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl Execute for Servo {
    async fn execute(&self, robot: Arc<Backends>) -> anyhow::Result<Self::Return> {
        let Servo(pin, deg) = *self;

        debug!("Servo basic: {deg}");

        #[cfg(feature = "backend")]
        if let Some(r) = &robot.raw_gpio {
            r.servo(pin, deg)?;
        }

        Ok(())
    }
}
