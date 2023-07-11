use anyhow::Result;
use roblib::{cmd::Command, event::Event};

// pub mod http;
// pub mod tcp;
pub mod udp;
// pub mod ws;

pub trait Transport {
    fn cmd<C>(&self, cmd: C) -> Result<C::Return>
    where
        C: Command,
        C::Return: Send;
}

pub trait Subscribable: Transport {
    fn subscribe<E, F>(&self, ev: E, handler: F) -> Result<()>
    where
        E: Event,
        F: FnMut(E::Item) -> Result<()>,
        F: Send + Sync + 'static;

    fn unsubscribe<E: Event>(&self, ev: E) -> Result<()>;
}

#[cfg(feature = "async")]
#[cfg_attr(feature = "async", async_trait::async_trait)]
pub trait TransportAsync: Send + Sync {
    async fn cmd<C>(&self, cmd: C) -> Result<C::Return>
    where
        C: Command + Send + Sync,
        C::Return: Send + Sync;
}

#[cfg(feature = "async")]
#[cfg_attr(feature = "async", async_trait::async_trait)]
pub trait SubscribableAsync: TransportAsync {
    async fn subscribe<E, F, R>(&self, ev: E, handler: F) -> Result<()>
    where
        E: Event + Send,
        E::Item: Send + Sync,
        F: FnMut(E::Item) -> R,
        F: Send + Sync + 'static,
        R: std::future::Future<Output = Result<()>> + Send + Sync;

    async fn unsubscribe<E: Event + Send>(&self, ev: E) -> Result<()>;
}
