use std::sync::Arc;

use crate::Robot;

use roblib::{
    cmd::{parsing::Writable, Command, Concrete, GetUptime, Nop},
    RoblibRobot,
};

#[cfg(feature = "roland")]
mod roland;

#[cfg(feature = "gpio")]
mod gpio;

#[cfg(feature = "camloc")]
mod camloc;

#[async_trait::async_trait]
pub(crate) trait Execute: Command {
    async fn execute(&self, robot: Arc<Robot>) -> anyhow::Result<Self::Return>;
}

pub(crate) async fn execute_concrete(
    concrete: Concrete,
    robot: Arc<Robot>,
) -> anyhow::Result<Option<Box<dyn Writable + Send + Sync>>> {
    type R = Box<dyn Writable + Send + Sync>;

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

impl RoblibRobot for Robot {
    fn nop(&self) -> anyhow::Result<()> {
        Ok(())
    }
    fn get_uptime(&self) -> anyhow::Result<std::time::Duration> {
        Ok(self.startup_time.elapsed())
    }
}

#[async_trait::async_trait]
impl Execute for Nop {
    async fn execute(&self, robot: Arc<Robot>) -> anyhow::Result<Self::Return> {
        debug!("Nop");
        robot.nop()
    }
}

#[async_trait::async_trait]
impl Execute for GetUptime {
    async fn execute(&self, robot: Arc<Robot>) -> anyhow::Result<Self::Return> {
        debug!("Get uptime");
        robot.get_uptime()
    }
}
