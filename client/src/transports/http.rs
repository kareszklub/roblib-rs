use awc::Client;
use roblib::{
    cmd::{Command, Concrete},
    Readable,
};
use roblib_parsing::SEPARATOR;

use crate::Transport;

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
    fn cmd<C: Command>(&self, cmd: C) -> anyhow::Result<C::Return>
    where
        C::Return: Readable,
    {
        self.runtime
            .block_on(send_cmd(&self.client, &self.base_url, cmd))
    }
}

#[cfg(feature = "async")]
pub struct HttpAsync {
    handle: futures::lock::Mutex<Option<tokio::task::JoinHandle<anyhow::Result<()>>>>,
    tx_cmd: futures::lock::Mutex<futures::channel::mpsc::UnboundedSender<Concrete>>,
    rx_res: futures::lock::Mutex<futures::channel::mpsc::UnboundedReceiver<String>>,
}

#[cfg(feature = "async")]
impl HttpAsync {
    pub async fn new(base_url: &str) -> anyhow::Result<HttpAsync> {
        let url = format!("{base_url}/cmd");

        let (tx_cmd, rx_cmd) = futures::channel::mpsc::unbounded();
        let (tx_res, rx_res) = futures::channel::mpsc::unbounded();

        Ok(Self {
            handle: Some(actix_rt::spawn(Self::run(url, rx_cmd, tx_res))).into(),
            tx_cmd: tx_cmd.into(),
            rx_res: rx_res.into(),
        })
    }

    async fn run(
        url: String,
        mut rx_cmd: futures::channel::mpsc::UnboundedReceiver<Concrete>,
        mut tx_res: futures::channel::mpsc::UnboundedSender<String>,
    ) -> anyhow::Result<()> {
        use futures_util::{SinkExt, StreamExt};

        let client = Client::default();

        while let Some(cmd) = rx_cmd.next().await {
            let req = client.post(&url).send_body(cmd.to_string());
            let mut res = match req.await {
                Ok(r) => r,
                Err(e) => return Err(anyhow::Error::msg(e.to_string())),
            };

            let body = res.body().await?;
            let body = String::from_utf8(body.to_vec())?;

            tx_res.send(body).await?;
        }

        Ok(())
    }
}

#[cfg(feature = "async")]
#[cfg_attr(feature = "async", async_trait::async_trait)]
impl super::TransportAsync for HttpAsync {
    async fn cmd_async<C: Command + Send>(&self, cmd: C) -> anyhow::Result<C::Return>
    where
        C::Return: Readable + Send,
    {
        use futures_util::{SinkExt, StreamExt};

        self.tx_cmd.lock().await.send(cmd.into()).await?;

        let Some(res) = self.rx_res.lock().await.next().await else {
            if let Some(r) = self.handle.lock().await.take() {
                r.await??;
                unreachable!("HTTP client terminated without error???")
            } else {
                return Err(anyhow::Error::msg("HTTP client already terminated"));
            }
        };

        Readable::parse_text(&mut res.split(SEPARATOR))
    }
}

async fn send_cmd<C: Command>(client: &Client, url: &str, cmd: C) -> anyhow::Result<C::Return>
where
    C::Return: Readable,
{
    let concrete: Concrete = cmd.into();

    let req = client.post(url).send_body(concrete.to_string()).await;
    let mut res = match req {
        Ok(x) => x,
        Err(e) => {
            return Err(anyhow::anyhow!(
                "didn't recieve Http response, because: {e}",
            ))
        }
    };

    let body = res.body().await?;
    let body = String::from_utf8(body.to_vec())?;

    Readable::parse_text(&mut body.split(SEPARATOR))
}
