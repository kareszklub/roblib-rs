extern crate log;

pub mod cmd;
pub mod event;
pub mod text_format;

#[cfg(feature = "camloc")]
pub mod camloc;

#[cfg(feature = "gpio")]
pub mod gpio;

#[cfg(feature = "roland")]
pub mod roland;

pub trait RoblibBuiltin {
    fn nop(&self) -> anyhow::Result<()>;
    fn get_uptime(&self) -> anyhow::Result<std::time::Duration>;
    fn abort(&self) -> anyhow::Result<()>;
}

#[cfg(feature = "async")]
#[async_trait::async_trait]
pub trait RoblibBuiltinAsync {
    async fn nop(&self) -> anyhow::Result<()>;
    async fn get_uptime(&self) -> anyhow::Result<std::time::Duration>;
    async fn abort(&self) -> anyhow::Result<()>;
}

fn map_num_range(v: f64, os: f64, oe: f64, ns: f64, ne: f64) -> f64 {
    let a = (v - os) / (oe - os);
    a * (ne - ns) + ns
}
