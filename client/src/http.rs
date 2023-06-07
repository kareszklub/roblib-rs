use actix_rt::Runtime;
use anyhow::{anyhow, Result};
use awc::Client;

use crate::RemoteRobotTransport;

pub struct RobotHTTP {
    base_url: String,
    client: Client,
    runtime: Runtime,
}

impl RobotHTTP {
    pub fn create(base_url: &str) -> Result<RobotHTTP> {
        Ok(Self {
            base_url: format!("{base_url}/cmd"),
            client: Client::default(),
            runtime: Runtime::new()?,
        })
    }

    /// Send a raw command.
    /// You probably don't need this.
    async fn send(&self, cmd: String) -> Result<Option<String>> {
        let mut req = match self.client.post(&self.base_url).send_body(cmd).await {
            Ok(x) => x,
            Err(e) => return Err(anyhow!("didn't recieve HTTP response, because: {e}")),
        };

        let body = req.body().await?;
        if body.is_empty() {
            return Ok(None);
        }

        Ok(Some(String::from_utf8(body.to_vec())?))
    }
}

impl RemoteRobotTransport for RobotHTTP {
    fn cmd(&self, cmd: roblib::cmd::Cmd) -> Result<Option<String>> {
        self.runtime.block_on(async {
            let s = cmd.to_string();
            debug!("S: {s}");

            let r = self.send(s).await?;
            if let Some(r) = &r {
                debug!("R: {r}");
            }

            Ok(r)
        })
    }
}
