#![allow(unused_imports, unused_variables)]
use roblib::{
    cmd::{ReadPin, ServoBasic, SetPin, SetPwm},
    gpio::Gpio,
};
use std::{future::Future, pin::Pin, sync::Arc};

use super::{Execute, Robot};

impl Execute for ReadPin {
    fn execute(
        &self,
        robot: Arc<Robot>,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<Self::Return>>>> {
        let ReadPin(pin) = *self;
        Box::pin(async move {
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
        })
    }
}

impl Execute for SetPin {
    fn execute(
        &self,
        robot: Arc<Robot>,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<Self::Return>>>> {
        let SetPin(pin, value) = *self;

        Box::pin(async move {
            debug!("Set pin: {pin}:{value}");

            #[cfg(feature = "backend")]
            if let Some(r) = &robot.raw_gpio {
                r.set_pin(pin, value)?;
            }

            Ok(())
        })
    }
}

impl Execute for SetPwm {
    fn execute(
        &self,
        robot: Arc<Robot>,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<Self::Return>>>> {
        let SetPwm(pin, hz, cycle) = *self;

        Box::pin(async move {
            debug!("Set pwm: {pin}:{hz}:{cycle}");

            #[cfg(feature = "backend")]
            if let Some(r) = &robot.raw_gpio {
                r.pwm(pin, hz, cycle)?;
            }

            Ok(())
        })
    }
}

impl Execute for ServoBasic {
    fn execute(
        &self,
        robot: Arc<Robot>,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<Self::Return>>>> {
        let ServoBasic(pin, deg) = *self;

        Box::pin(async move {
            debug!("Servo basic: {deg}");

            #[cfg(feature = "backend")]
            if let Some(r) = &robot.raw_gpio {
                r.servo(pin, deg)?;
            }

            Ok(())
        })
    }
}
