use std::{future::Future, io::Read, pin::Pin, sync::Arc};

use crate::Robot;

use anyhow::Result;
use roblib::{
    cmd::{
        parsing::{commands::Concrete, Writable},
        Command, GetUptime, Nop,
    },
    RoblibRobot,
};

#[cfg(feature = "roland")]
mod roland;

#[cfg(feature = "gpio")]
mod gpio;

#[cfg(feature = "camloc")]
mod camloc;

pub(crate) trait Execute: Command {
    fn execute(
        &self,
        robot: Arc<Robot>,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<Self::Return>>>>;
}

async fn execute_concrete(
    concrete: Concrete,
    robot: Arc<Robot>,
) -> anyhow::Result<Option<Box<dyn Writable + Send>>> {
    type R = Box<dyn Writable + Send>;

    Ok(match concrete {
        #[cfg(feature = "roland")]
        Concrete::MoveRobot(c) => {
            c.execute(robot).await?;
            None
        }
        #[cfg(feature = "roland")]
        Concrete::MoveRobotByAngle(c) => {
            c.execute(robot).await?;
            None
        }
        #[cfg(feature = "roland")]
        Concrete::StopRobot(c) => {
            c.execute(robot).await?;
            None
        }
        #[cfg(feature = "roland")]
        Concrete::Led(c) => {
            c.execute(robot).await?;
            None
        }
        #[cfg(feature = "roland")]
        Concrete::ServoAbsolute(c) => {
            c.execute(robot).await?;
            None
        }
        #[cfg(feature = "roland")]
        Concrete::Buzzer(c) => {
            c.execute(robot).await?;
            None
        }
        #[cfg(feature = "roland")]
        Concrete::TrackSensor(c) => Some(Box::new(c.execute(robot).await?) as R),
        #[cfg(feature = "roland")]
        Concrete::UltraSensor(c) => Some(Box::new(c.execute(robot).await?) as R),

        #[cfg(feature = "gpio")]
        Concrete::ReadPin(c) => Some(Box::new(c.execute(robot).await?) as R),
        #[cfg(feature = "gpio")]
        Concrete::SetPin(c) => {
            c.execute(robot).await?;
            None
        }
        #[cfg(feature = "gpio")]
        Concrete::SetPwm(c) => {
            c.execute(robot).await?;
            None
        }
        #[cfg(feature = "gpio")]
        Concrete::ServoBasic(c) => {
            c.execute(robot).await?;
            None
        }

        #[cfg(feature = "camloc")]
        Concrete::GetPosition(c) => Some(Box::new(c.execute(robot).await?) as R),

        Concrete::Nop(c) => {
            c.execute(robot).await?;
            None
        }
        Concrete::GetUptime(c) => Some(Box::new(c.execute(robot).await?) as R),
    })
}

#[allow(unused)]
pub(crate) async fn execute_command_text<'a>(
    input: &mut impl Iterator<Item = &'a str>,
    robot: Arc<Robot>,
) -> Result<Option<Box<dyn Writable + Send>>> {
    execute_concrete(Concrete::parse_str(input)?, robot).await
}
#[allow(unused)]
pub(crate) async fn execute_command_binary(
    input: &mut impl Read,
    robot: Arc<Robot>,
) -> Result<Option<Box<dyn Writable + Send>>> {
    execute_concrete(Concrete::parse_binary(input)?, robot).await
}

impl RoblibRobot for Robot {
    fn nop(&self) -> anyhow::Result<()> {
        Ok(())
    }
    fn get_uptime(&self) -> anyhow::Result<std::time::Duration> {
        Ok(self.startup_time.elapsed())
    }
}

impl Execute for Nop {
    fn execute(
        &self,
        robot: Arc<Robot>,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<Self::Return>>>> {
        Box::pin(async move {
            debug!("Nop");
            robot.nop()
        })
    }
}

impl Execute for GetUptime {
    fn execute(
        &self,
        robot: Arc<Robot>,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<Self::Return>>>> {
        Box::pin(async move {
            debug!("Get uptime");
            robot.get_uptime()
        })
    }
}
