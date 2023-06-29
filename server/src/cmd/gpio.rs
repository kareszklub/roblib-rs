#![allow(unused_imports, unused_variables)]
use roblib::{
    cmd::{ReadPin, ServoBasic, SetPin, SetPwm},
    gpio::Gpio,
};
use std::sync::Arc;

use super::{Execute, Robot};

#[async_trait::async_trait]
impl Execute for ReadPin {
    async fn execute(&self, robot: Arc<Robot>) -> anyhow::Result<Self::Return> {
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
impl Execute for SetPin {
    async fn execute(&self, robot: Arc<Robot>) -> anyhow::Result<Self::Return> {
        let SetPin(pin, value) = *self;

        debug!("Set pin: {pin}:{value}");

        #[cfg(feature = "backend")]
        if let Some(r) = &robot.raw_gpio {
            r.set_pin(pin, value)?;
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl Execute for SetPwm {
    async fn execute(&self, robot: Arc<Robot>) -> anyhow::Result<Self::Return> {
        let SetPwm(pin, hz, cycle) = *self;

        debug!("Set pwm: {pin}:{hz}:{cycle}");

        #[cfg(feature = "backend")]
        if let Some(r) = &robot.raw_gpio {
            r.pwm(pin, hz, cycle)?;
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl Execute for ServoBasic {
    async fn execute(&self, robot: Arc<Robot>) -> anyhow::Result<Self::Return> {
        let ServoBasic(pin, deg) = *self;

        debug!("Servo basic: {deg}");

        #[cfg(feature = "backend")]
        if let Some(r) = &robot.raw_gpio {
            r.servo(pin, deg)?;
        }

        Ok(())
    }
}
