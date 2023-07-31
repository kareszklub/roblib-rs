use std::sync::Arc;

use crate::Backends;

use roblib::cmd::{Abort, Command, Concrete, GetUptime, Nop};
use serde::{Serialize, Serializer};

#[cfg(feature = "roland")]
mod roland;

#[cfg(feature = "gpio")]
mod gpio;

#[cfg(feature = "camloc")]
mod camloc;

#[async_trait::async_trait]
pub(crate) trait Execute: Command {
    async fn execute(&self, robot: Arc<Backends>) -> anyhow::Result<Self::Return>;
}

pub(crate) async fn execute_concrete<S>(
    concrete: Concrete,
    robot: Arc<Backends>,
    ser: S,
) -> anyhow::Result<Option<S::Ok>>
where
    S: Serializer + Send,
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
        Concrete::RolandServo(c) => {
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
        Concrete::PinMode(c) => {
            c.execute(robot).await?;
            None
        }
        #[cfg(feature = "gpio")]
        Concrete::ReadPin(c) => Some(
            c.execute(robot)
                .await?
                .serialize(ser)
                .map_err(|e| anyhow::Error::msg(e.to_string()))?,
        ),
        #[cfg(feature = "gpio")]
        Concrete::WritePin(c) => {
            c.execute(robot).await?;
            None
        }
        #[cfg(feature = "gpio")]
        Concrete::Pwm(c) => {
            c.execute(robot).await?;
            None
        }
        #[cfg(feature = "gpio")]
        Concrete::Servo(c) => {
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

        Concrete::Subscribe(_) => unreachable!("Subscribe should be handled by the transport"),
        Concrete::Unsubscribe(_) => unreachable!("Unsubscribe should be handled by the transport"),

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
        Concrete::Abort(c) => {
            c.execute(robot).await?;
            None
        }
    })
}

#[async_trait::async_trait]
impl Execute for Nop {
    async fn execute(&self, _: Arc<Backends>) -> anyhow::Result<Self::Return> {
        debug!("Nop");
        Ok(())
    }
}

#[async_trait::async_trait]
impl Execute for GetUptime {
    async fn execute(&self, robot: Arc<Backends>) -> anyhow::Result<Self::Return> {
        debug!("Get uptime");
        Ok(robot.startup_time.elapsed())
    }
}

#[async_trait::async_trait]
impl Execute for Abort {
    async fn execute(&self, robot: Arc<Backends>) -> anyhow::Result<Self::Return> {
        error!("Abort");
        robot.abort_token.cancel();
        Ok(())
    }
}
