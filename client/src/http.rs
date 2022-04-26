use anyhow::{anyhow, Result};
use awc::Client;

pub struct Robot {
    base_url: String,
    client: Client,
}
impl Robot {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: format!("{base_url}/cmd"),
            client: Client::default(),
        }
    }

    async fn send(&self, cmd: &'static str) -> Result<String> {
        let mut req = match self.client.post(&self.base_url).send_body(cmd).await {
            Ok(x) => x,
            Err(_) => return Err(anyhow!("failed to connect")),
        };

        Ok(String::from_utf8(req.body().await?.to_vec())?)
    }

    pub async fn stop_robot(&self) -> Result<String> {
        debug!("S: s");
        let res = self.send("s").await?;
        debug!("R: {res}");
        Ok(res)
    }
}
