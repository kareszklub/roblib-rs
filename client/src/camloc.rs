use anyhow::Result;
use roblib::cmd;

use crate::{transports::Transport, Robot};

impl<T: Transport> roblib::camloc::Camloc for Robot<T> {
    fn get_position(&self) -> Result<Option<roblib::camloc::Position>> {
        self.transport.cmd(cmd::GetPosition)
    }
}

#[cfg(feature = "async")]
#[async_trait::async_trait]
impl<T: crate::transports::TransportAsync> roblib::camloc::CamlocAsync
    for crate::async_robot::RobotAsync<T>
{
    async fn get_position(&self) -> Result<Option<roblib::camloc::Position>> {
        self.transport.cmd(cmd::GetPosition).await
    }
}
