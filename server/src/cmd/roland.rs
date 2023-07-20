#![allow(unused_imports, unused_variables)]
use std::sync::Arc;

use roblib::{
    cmd::{
        Buzzer, Led, MoveRobot, MoveRobotByAngle, ServoAbsolute, StopRobot, TrackSensor,
        UltraSensor,
    },
    roland::Roland,
};

use super::{Backends, Execute};

#[async_trait::async_trait]
impl Execute for MoveRobot {
    async fn execute(&self, robot: Arc<Backends>) -> anyhow::Result<Self::Return> {
        let MoveRobot(left, right) = *self;

        debug!("Moving robot: {left}:{right}");

        #[cfg(feature = "backend")]
        if let Some(r) = &robot.roland {
            r.drive(left, right)?;

            #[cfg(feature = "camloc")]
            if let Some(c) = &robot.camloc {
                let hint = roblib::camloc::get_motion_hint(left, right);
                c.service.set_motion_hint(hint).await;
            }
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl Execute for MoveRobotByAngle {
    async fn execute(&self, robot: Arc<Backends>) -> anyhow::Result<Self::Return> {
        let MoveRobotByAngle(angle, speed) = *self;

        debug!("Moving robot by angle: {}:{}", angle, speed);

        #[cfg(feature = "backend")]
        if let Some(r) = &robot.roland {
            r.drive_by_angle(angle, speed)?;

            #[cfg(feature = "camloc")]
            if let Some(c) = &robot.camloc {
                let (left, right) = roblib::roland::convert_move(angle, speed);
                let hint = roblib::camloc::get_motion_hint(left, right);
                c.service.set_motion_hint(hint).await;
            }
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl Execute for StopRobot {
    async fn execute(&self, robot: Arc<Backends>) -> anyhow::Result<Self::Return> {
        debug!("Stopping robot");

        #[cfg(feature = "backend")]
        if let Some(r) = &robot.roland {
            r.drive(0., 0.)?;
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl Execute for Led {
    async fn execute(&self, robot: Arc<Backends>) -> anyhow::Result<Self::Return> {
        let Led(r, g, b) = *self;

        debug!("LED: {r}:{g}:{b}");

        #[cfg(feature = "backend")]
        if let Some(rr) = &robot.roland {
            rr.led(r, g, b)?;
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl Execute for ServoAbsolute {
    async fn execute(&self, robot: Arc<Backends>) -> anyhow::Result<Self::Return> {
        let ServoAbsolute(deg) = *self;

        debug!("Servo absolute: {deg}");

        #[cfg(feature = "backend")]
        if let Some(r) = &robot.roland {
            r.servo(deg)?;
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl Execute for Buzzer {
    async fn execute(&self, robot: Arc<Backends>) -> anyhow::Result<Self::Return> {
        let Buzzer(pw) = *self;

        debug!("Buzzer: {pw}");

        #[cfg(feature = "backend")]
        if let Some(r) = &robot.roland {
            r.buzzer(pw)?
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl Execute for TrackSensor {
    async fn execute(&self, robot: Arc<Backends>) -> anyhow::Result<Self::Return> {
        debug!("Track sensor");

        #[cfg(feature = "backend")]
        let res = if let Some(r) = &robot.roland {
            r.track_sensor()?
        } else {
            [false, false, false, false]
        };

        #[cfg(not(feature = "backend"))]
        let res = [false, false, false, false];

        Ok(res)
    }
}

#[async_trait::async_trait]
impl Execute for UltraSensor {
    async fn execute(&self, robot: Arc<Backends>) -> anyhow::Result<Self::Return> {
        debug!("Ultra sensor");

        #[cfg(feature = "backend")]
        let res = if robot.roland.is_some() {
            // because it uses std::thread::sleep
            actix_web::rt::task::spawn_blocking(move || {
                robot.roland.as_ref().unwrap().ultra_sensor()
            })
            .await??
        } else {
            f64::NAN
        };

        #[cfg(not(feature = "backend"))]
        let res = f64::NAN;

        Ok(res)
    }
}
