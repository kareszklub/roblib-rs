#[macro_use]
extern crate log;

pub mod cmd;

#[cfg(all(unix, feature = "gpio"))]
pub mod gpio;

#[cfg(all(not(unix), feature = "gpio"))]
pub mod gpio {
    pub fn try_init() -> anyhow::Result<()> {
        Err(anyhow::anyhow!("unsupported platform"))
    }
}
