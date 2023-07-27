use crate::Transport;
use awc::Client;
use roblib::cmd::{self, Command};

pub struct Http {
    base_url: String,
    client: Client,
    runtime: actix_rt::Runtime,
}

impl Http {
    pub fn new(base_url: &str) -> anyhow::Result<Self> {
        Ok(Self {
            base_url: format!("{base_url}/cmd"),
            client: Client::default(),
            runtime: actix_rt::Runtime::new()?,
        })
    }
}

impl Transport for Http {
    fn cmd<C>(&self, cmd: C) -> anyhow::Result<C::Return>
    where
        C: Command,
        C::Return: Send + 'static,
    {
        let cmd = cmd::concrete::Concr::new(cmd.into());

        let body = self.runtime.block_on(async {
            let req = self
                .client
                .post(&self.base_url)
                .send_body(cmd.to_string())
                .await;

            let mut res = match req {
                Ok(x) => x,
                Err(e) => {
                    return Err(anyhow::anyhow!(
                        "didn't recieve Http response, because: {e}",
                    ))
                }
            };

            Ok(res.body().await?)
        })?;

        let body = String::from_utf8(body.to_vec())?;

        Ok(roblib::text_format::de::from_str(&body)?)
    }
}
