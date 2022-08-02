mod constants;
#[cfg(feature = "roland")]
#[cfg(unix)]
pub mod roland;

#[cfg(feature = "roland")]
#[cfg(not(unix))]
pub mod roland {
    pub fn try_init() -> anyhow::Result<()> {
        Err(anyhow::anyhow!("unsupported platform"))
    }
}
