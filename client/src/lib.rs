pub mod logger;
pub mod transports;

pub use anyhow::Result;
pub use roblib;

#[cfg(feature = "async")]
pub use tokio::main;
#[cfg(feature = "async")]
pub mod async_robot;
#[cfg(feature = "async")]
pub use async_robot::RobotAsync;

#[cfg(feature = "camloc")]
mod camloc;
#[cfg(feature = "gpio")]
mod gpio;
#[cfg(feature = "roland")]
mod roland;

use roblib::{cmd, event::Event, RoblibBuiltin};
use std::time::{Duration, Instant};
use transports::{Subscribable, Transport};

pub struct Robot<T> {
    pub transport: T,
}

impl<T: Transport> Robot<T> {
    pub fn new(transport: T) -> Self {
        Self { transport }
    }

    pub fn measure_latency(&self) -> Result<Duration> {
        let start = Instant::now();
        let _ = self.transport.cmd(cmd::GetUptime)?;
        Ok(Instant::now() - start)
    }
    pub fn get_server_uptime(&self) -> Result<Duration> {
        self.transport.cmd(cmd::GetUptime)
    }
}
impl<T: Subscribable> Robot<T> {
    pub fn subscribe<E: Event>(
        &self,
        ev: E,
        handler: impl FnMut(E::Item) -> Result<()> + Send + Sync + 'static,
    ) -> Result<()> {
        self.transport.subscribe(ev, handler)
    }
    pub fn unsubscribe<E: Event>(&self, ev: E) -> Result<()> {
        self.transport.unsubscribe(ev)
    }
}

impl<T: Transport> RoblibBuiltin for Robot<T> {
    fn nop(&self) -> anyhow::Result<()> {
        self.transport.cmd(cmd::Nop)
    }

    fn get_uptime(&self) -> anyhow::Result<Duration> {
        self.transport.cmd(cmd::GetUptime)
    }

    fn abort(&self) -> anyhow::Result<()> {
        self.transport.cmd(cmd::Abort)
    }
}
