use std::time::{Duration, Instant};

use anyhow::Result;
use roblib::{
    cmd,
    cmd::{parsing::Readable, Command},
};

pub mod http;
pub mod tcp;
pub mod udp;
pub mod ws;

pub trait Transport {
    fn cmd<C: Command>(&self, cmd: C) -> Result<C::Return>
    where
        C::Return: Readable;

    fn measure_latency(&self) -> Result<Duration> {
        let start = Instant::now();
        self.cmd(cmd::GetUptime)?;
        Ok(Instant::now() - start)
    }

    fn get_server_uptime(&self) -> Result<Duration> {
        self.cmd(cmd::GetUptime)
    }
}

#[cfg(feature = "async")]
#[cfg_attr(feature = "async", async_trait::async_trait)]
pub trait TransportAsync: Send + Sync {
    async fn cmd_async<C: Command + Send>(&self, cmd: C) -> Result<C::Return>
    where
        C::Return: Readable + Send;

    async fn measure_latency(&self) -> Result<Duration> {
        let start = Instant::now();
        self.cmd_async(cmd::GetUptime).await?;
        Ok(Instant::now() - start)
    }

    async fn get_server_uptime(&self) -> Result<Duration> {
        self.cmd_async(cmd::GetUptime).await
    }
}
