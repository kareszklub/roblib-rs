use std::{future::Future, pin::Pin, sync::Arc};

use roblib::cmd::GetPosition;

use super::{Execute, Robot};

impl Execute for GetPosition {
    fn execute(
        &self,
        robot: Arc<Robot>,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<Self::Return>>>> {
        Box::pin(async move {
            debug!("Get position");
            Ok(if let Some(c) = &robot.camloc {
                c.service.get_position().await.map(|tp| tp.position)
            } else {
                None
            })
        })
    }
}
