use anyhow::{anyhow, Result};
use awc::Client;
use futures::executor::block_on;

use crate::RemoteRobotTransport;

pub struct RobotHTTP {
    base_url: String,
    client: Client,
}
impl RobotHTTP {
    pub async fn connect(base_url: &str) -> Result<Self> {
        Ok(Self {
            base_url: format!("{base_url}/cmd"),
            client: Client::default(),
        })
    }

    /// Send a raw command.
    /// You probably don't need this.
    pub async fn send(&self, cmd: String) -> Result<String> {
        let mut req = match self.client.post(&self.base_url).send_body(cmd).await {
            Ok(x) => x,
            Err(_) => return Err(anyhow!("failed to connect")),
        };

        Ok(String::from_utf8(req.body().await?.to_vec())?)
    }
}

impl RemoteRobotTransport for RobotHTTP {
    fn cmd(&self, cmd: roblib::cmd::Cmd) -> Result<String> {
        block_on(async {
            let s = cmd.to_string();
            debug!("S: {}", &s);
            let r = self.send(s).await?;
            debug!("R: {}", &r);
            Ok(r)
        })
    }
}
