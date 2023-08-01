use reqwest::Client;
use roblib::{
    cmd::{has_return, Command, Concrete},
    text_format,
};

pub struct Http {
    base_url: String,
    client: Client,
}

impl Http {
    pub fn connect(base_url: &str) -> anyhow::Result<Self> {
        Ok(Self {
            base_url: format!("http://{base_url}/cmd"),
            client: Client::new(),
        })
    }
}

#[cfg(feature = "async")]
#[cfg_attr(feature = "async", async_trait::async_trait)]
impl super::TransportAsync for Http {
    async fn cmd<C: Command + Send>(&self, cmd: C) -> anyhow::Result<C::Return> {
        let cmd: Concrete = cmd.into();
        let s = text_format::ser::to_string(&cmd)?;

        dbg!(&s);

        let res = self
            .client
            .post(&self.base_url)
            .body(s)
            .send()
            .await?
            .text()
            .await?;

        dbg!(&res);

        if has_return::<C>() {
            Ok(text_format::de::from_str(&res)?)
        } else {
            Ok(unsafe { std::mem::zeroed() })
        }
    }
}
