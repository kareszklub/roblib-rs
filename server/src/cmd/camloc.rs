use std::sync::Arc;

use roblib::cmd::GetPosition;

use super::{Execute, Robot};

#[async_trait::async_trait]
impl Execute for GetPosition {
    async fn execute(&self, robot: Arc<Robot>) -> anyhow::Result<Self::Return> {
        debug!("Get position");
        Ok(if let Some(c) = &robot.camloc {
            c.service.get_position().await
        } else {
            None
        })
    }
}
