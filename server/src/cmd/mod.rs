use std::sync::Arc;

use crate::Robot;

use roblib::{
    cmd::{Command, Concrete, GetUptime, Nop, Subscribe, Unsubscribe},
    RoblibRobot,
};
use serde::{Serialize, Serializer};

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

pub(crate) async fn execute_concrete<S>(
    concrete: Concrete,
    robot: Arc<Robot>,
    ser: S,
) -> anyhow::Result<Option<S::Ok>>
where
    S: Serializer + Send,
    S::Error: Send,
{
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
        Concrete::TrackSensor(c) => Some(
            c.execute(robot)
                .await?
                .serialize(ser)
                .map_err(|e| anyhow::Error::msg(e.to_string()))?,
        ),
        #[cfg(feature = "roland")]
        Concrete::UltraSensor(c) => Some(
            c.execute(robot)
                .await?
                .serialize(ser)
                .map_err(|e| anyhow::Error::msg(e.to_string()))?,
        ),

        #[cfg(feature = "gpio")]
        Concrete::ReadPin(c) => Some(
            c.execute(robot)
                .await?
                .serialize(ser)
                .map_err(|e| anyhow::Error::msg(e.to_string()))?,
        ),
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
        Concrete::GetPosition(c) => Some(
            c.execute(robot)
                .await?
                .serialize(ser)
                .map_err(|e| anyhow::Error::msg(e.to_string()))?,
        ),

        Concrete::Subscribe(c) => Some(
            c.execute(robot)
                .await?
                .serialize(ser)
                .map_err(|e| anyhow::Error::msg(e.to_string()))?,
        ),
        Concrete::Unsubscribe(c) => Some(
            c.execute(robot)
                .await?
                .serialize(ser)
                .map_err(|e| anyhow::Error::msg(e.to_string()))?,
        ),

        Concrete::Nop(c) => {
            c.execute(robot).await?;
            None
        }
        Concrete::GetUptime(c) => Some(
            c.execute(robot)
                .await?
                .serialize(ser)
                .map_err(|e| anyhow::Error::msg(e.to_string()))?,
        ),
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
impl Execute for Subscribe {
    async fn execute(&self, robot: Arc<Robot>) -> anyhow::Result<Self::Return> {
        debug!("Subscribe");
        todo!()
    }
}

#[async_trait::async_trait]
impl Execute for Unsubscribe {
    async fn execute(&self, robot: Arc<Robot>) -> anyhow::Result<Self::Return> {
        debug!("Unsubscribe");
        todo!()
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
