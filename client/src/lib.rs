#[macro_use]
extern crate log;
pub use roblib_shared::{cmd, logger};
pub mod http;
pub mod ws;
pub use anyhow::Result;
