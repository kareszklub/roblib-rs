pub mod cmd;
pub mod event;

pub use camloc_server::{service, MotionHint, Position};

pub trait Camloc {
    fn get_position(&self) -> anyhow::Result<Option<Position>>;
}

#[cfg(feature = "async")]
#[async_trait::async_trait]
pub trait CamlocAsync {
    async fn get_position(&self) -> anyhow::Result<Option<Position>>;
}

#[cfg(all(feature = "roland", feature = "gpio-backend"))]
pub fn get_motion_hint(left: f64, right: f64) -> Option<MotionHint> {
    let left_sign = left.signum() as isize;
    let right_sign = right.signum() as isize;

    match (left_sign, right_sign) {
        (1, 1) | (1, 0) | (0, 1) => Some(MotionHint::MovingForwards),

        (0, 0) => Some(MotionHint::Stationary),

        (-1, -1) | (-1, 0) | (0, -1) => Some(MotionHint::MovingBackwards),

        // turning in place
        (1, -1) | (-1, 1) if (left * 100.) as usize == (-right * 100.) as usize => None,

        _ => None,
    }
}
