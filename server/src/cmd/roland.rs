use std::{future::Future, pin::Pin, sync::Arc};

use roblib::{
    cmd::{
        Buzzer, Led, MoveRobot, MoveRobotByAngle, ServoAbsolute, StopRobot, TrackSensor,
        UltraSensor,
    },
    roland::Roland,
};

use super::{Execute, Robot};

impl Execute for MoveRobot {
    fn execute(
        &self,
        robot: Arc<Robot>,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<Self::Return>>>> {
        let MoveRobot(left, right) = *self;

        Box::pin(async move {
            debug!("Moving robot: {left}:{right}");

            #[cfg(feature = "backend")]
            if let Some(r) = &robot.roland {
                #[allow(clippy::let_unit_value)]
                let hint = r.drive(left, right)?;

                #[cfg(feature = "camloc")]
                if let Some(c) = &robot.camloc {
                    c.service.set_motion_hint(hint).await;
                }

                Ok(hint)
            } else {
                #[cfg(feature = "camloc")]
                let ret = None;
                #[cfg(not(feature = "camloc"))]
                let ret = ();

                Ok(ret)
            }
        })
    }
}

impl Execute for MoveRobotByAngle {
    fn execute(
        &self,
        robot: Arc<Robot>,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<Self::Return>>>> {
        let MoveRobotByAngle(angle, speed) = *self;

        Box::pin(async move {
            debug!("Moving robot by angle: {}:{}", angle, speed);

            #[cfg(feature = "backend")]
            if let Some(r) = &robot.roland {
                #[allow(clippy::let_unit_value)]
                let hint = r.drive_by_angle(angle, speed)?;

                #[cfg(feature = "camloc")]
                if let Some(c) = &robot.camloc {
                    c.service.set_motion_hint(hint).await;
                }
                Ok(hint)
            } else {
                #[cfg(feature = "camloc")]
                let ret = None;
                #[cfg(not(feature = "camloc"))]
                let ret = ();

                Ok(ret)
            }
        })
    }
}

impl Execute for StopRobot {
    fn execute(
        &self,
        robot: Arc<Robot>,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<Self::Return>>>> {
        Box::pin(async move {
            debug!("Stopping robot");

            #[cfg(feature = "backend")]
            if let Some(r) = &robot.roland {
                r.drive(0., 0.)?;
            }
            Ok(())
        })
    }
}

impl Execute for Led {
    fn execute(
        &self,
        robot: Arc<Robot>,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<Self::Return>>>> {
        let Led(r, g, b) = *self;

        Box::pin(async move {
            debug!("LED: {r}:{g}:{b}");

            #[cfg(feature = "backend")]
            if let Some(rr) = &robot.roland {
                rr.led(r, g, b)?;
            }
            Ok(())
        })
    }
}

impl Execute for ServoAbsolute {
    fn execute(
        &self,
        robot: Arc<Robot>,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<Self::Return>>>> {
        let ServoAbsolute(deg) = *self;

        Box::pin(async move {
            debug!("Servo absolute: {deg}");

            #[cfg(feature = "backend")]
            if let Some(r) = &robot.roland {
                r.servo(deg)?;
            }

            Ok(())
        })
    }
}

impl Execute for Buzzer {
    fn execute(
        &self,
        robot: Arc<Robot>,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<Self::Return>>>> {
        let Buzzer(pw) = *self;

        Box::pin(async move {
            debug!("Buzzer: {pw}");

            #[cfg(feature = "backend")]
            if let Some(r) = &robot.roland {
                r.buzzer(pw)?
            }
            Ok(())
        })
    }
}

impl Execute for TrackSensor {
    fn execute(
        &self,
        robot: Arc<Robot>,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<Self::Return>>>> {
        Box::pin(async move {
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
        })
    }
}

impl Execute for UltraSensor {
    fn execute(
        &self,
        robot: Arc<Robot>,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<Self::Return>>>> {
        Box::pin(async move {
            debug!("Ultra sensor");

            #[cfg(feature = "backend")]
            let res = if let Some(r) = &robot.roland {
                r.ultra_sensor()?
            } else {
                f64::NAN
            };

            #[cfg(not(feature = "backend"))]
            let res = f64::NAN;

            Ok(res)
        })
    }
}
