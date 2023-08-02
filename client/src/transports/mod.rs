use anyhow::Result;
use roblib::{cmd::Command, event::Event};

#[cfg(feature = "http")]
pub mod http;
#[cfg(feature = "tcp")]
pub mod tcp;
#[cfg(feature = "udp")]
pub mod udp;
#[cfg(feature = "ws")]
pub mod ws;

const ID_START: u32 = 1;

pub trait Transport {
    fn cmd<C>(&self, cmd: C) -> Result<C::Return>
    where
        C: Command;
}

pub trait Subscribable: Transport {
    fn subscribe<E, F>(&self, event: E, handler: F) -> Result<()>
    where
        E: Event,
        F: (FnMut(E::Item) -> Result<()>) + Send + Sync + 'static;

    fn unsubscribe<E>(&self, event: E) -> Result<()>
    where
        E: Event;
}

#[cfg(feature = "async")]
#[cfg_attr(feature = "async", async_trait::async_trait)]
pub trait TransportAsync: Send + Sync {
    async fn cmd<C>(&self, cmd: C) -> Result<C::Return>
    where
        C: Command;
}

#[cfg(feature = "async")]
#[cfg_attr(feature = "async", async_trait::async_trait)]
pub trait SubscribableAsync: TransportAsync {
    async fn subscribe<E: Event>(&self, ev: E)
        -> Result<tokio::sync::broadcast::Receiver<E::Item>>;

    async fn unsubscribe<E>(&self, ev: E) -> Result<()>
    where
        E: Event;
}
