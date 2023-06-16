use actix_rt::Runtime;
use awc::Client;
use roblib::cmd::{parsing::Readable, Command, SEPARATOR};

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

    async fn send<C: Command>(&self, cmd: C) -> anyhow::Result<C::Return>
    where
        C::Return: Readable,
    {
        let mut out = C::PREFIX.to_string();
        cmd.write_str(&mut |r| {
            out.push(SEPARATOR);
            out.push_str(r);
        })?;

        let req = self
            .client
            .post(&self.base_url)
            .send_body(out.to_string())
            .await;

        let mut res = match req {
            Ok(x) => x,
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "didn't recieve HTTP response, because: {e}",
                ))
            }
        };

        let body = res.body().await?;
        let body = String::from_utf8(body.to_vec())?;

        Readable::parse_str(&mut body.split(SEPARATOR))
    }
}

impl RemoteRobotTransport for RobotHTTP {
    fn cmd<C: Command>(&self, cmd: C) -> anyhow::Result<C::Return>
    where
        C::Return: Readable,
    {
        self.runtime.block_on(self.send(cmd))
    }
}
