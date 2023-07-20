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
}

#[cfg(feature = "async")]
#[async_trait::async_trait]
pub trait RoblibBuiltinAsync {
    async fn nop(&self) -> anyhow::Result<()>;
    async fn get_uptime(&self) -> anyhow::Result<std::time::Duration>;
}

#[allow(unused)]
pub(crate) fn get_servo_pwm_durations(degree: f64) -> (std::time::Duration, std::time::Duration) {
    let degree = ((degree.clamp(-90., 90.) as i64 + 90) as u64 * 11) + 500;
    (
        std::time::Duration::from_millis(20),
        std::time::Duration::from_micros(degree),
    ) // 50Hz
}
