use crate::transports::{SubscribableAsync, TransportAsync};
use anyhow::Result;
use async_trait::async_trait;
use roblib::{cmd, event::Event, RoblibBuiltinAsync};
use std::time::{Duration, Instant};

pub struct RobotAsync<T> {
    pub transport: T,
}

impl<T: TransportAsync> RobotAsync<T> {
    pub fn new(transport: T) -> Self {
        Self { transport }
    }

    pub async fn measure_latency(&self) -> Result<Duration> {
        let start = Instant::now();
        let _ = self.transport.cmd(cmd::GetUptime).await?;
        Ok(Instant::now() - start)
    }
    pub async fn get_server_uptime(&self) -> Result<Duration> {
        self.transport.cmd(cmd::GetUptime).await
    }
}
impl<T: SubscribableAsync> RobotAsync<T> {
    pub async fn subscribe<E, F, R>(&self, ev: E, handler: F) -> Result<()>
    where
        E: Event,
        F: (FnMut(E::Item) -> R) + Send + Sync + 'static,
        R: std::future::Future<Output = Result<()>> + Send + Sync,
    {
        self.transport.subscribe(ev, handler).await
    }
    pub async fn unsubscribe<E: Event>(&self, ev: E) -> Result<()> {
        self.transport.unsubscribe(ev).await
    }
}

#[async_trait]
impl<T: TransportAsync> RoblibBuiltinAsync for RobotAsync<T> {
    async fn nop(&self) -> anyhow::Result<()> {
        self.transport.cmd(cmd::Nop).await
    }

    async fn get_uptime(&self) -> anyhow::Result<Duration> {
        self.transport.cmd(cmd::GetUptime).await
    }

    async fn abort(&self) -> anyhow::Result<()> {
        self.transport.cmd(cmd::Abort).await
    }
}
