use actix_rt::Runtime;
use anyhow::anyhow;
use awc::Client;

use crate::RemoteRobotTransport;

pub struct RobotHTTP {
    base_url: String,
    client: Client,
    runtime: Runtime,
}

impl RobotHTTP {
    pub fn create(base_url: &str) -> anyhow::Result<RobotHTTP> {
        Ok(Self {
            base_url: format!("{base_url}/cmd"),
            client: Client::default(),
            runtime: Runtime::new()?,
        })
    }

    async fn send(&self, cmd: String, wait_for_response: bool) -> anyhow::Result<Option<String>> {
        let mut req = match self.client.post(&self.base_url).send_body(cmd).await {
            Ok(x) => x,
            Err(e) => return Err(anyhow!("didn't recieve HTTP response, because: {e}")),
        };

        if !wait_for_response {
            return Ok(None);
        }

        let body = req.body().await?;
        if body.is_empty() {
            return Ok(None);
        }

        Ok(Some(String::from_utf8(body.to_vec())?))
    }
}

impl RemoteRobotTransport for RobotHTTP {
    fn cmd(&self, cmd: roblib::cmd::Cmd) -> anyhow::Result<Option<String>> {
        self.runtime.block_on(async {
            let s = cmd.to_string();
            debug!("S: {s}");

            let r = self.send(s, cmd.has_return()).await?;
            if let Some(r) = &r {
                debug!("R: {r}");
            }

            Ok(r)
        })
    }
}
