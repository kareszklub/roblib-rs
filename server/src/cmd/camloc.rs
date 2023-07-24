use std::sync::Arc;

use roblib::cmd::GetPosition;

use super::{Backends, Execute};

#[async_trait::async_trait]
impl Execute for GetPosition {
    async fn execute(&self, robot: Arc<Backends>) -> anyhow::Result<Self::Return> {
        debug!("Get position");

        #[cfg(feature = "backend")]
        if let Some(c) = &robot.camloc {
            return Ok(c.get_position().await);
        }

        Ok(None)
    }
}
