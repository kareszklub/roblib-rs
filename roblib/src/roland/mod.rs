use anyhow::Result;

#[cfg(feature = "gpio-backend")]
pub mod backend;

#[cfg(feature = "camloc")]
pub type DriveResult = Option<crate::camloc::MotionHint>;
#[cfg(not(feature = "camloc"))]
pub type DriveResult = ();

#[cfg(feature = "gpio-backend")]
pub trait Roland: Sized {
    fn drive(&self, left: f64, right: f64) -> Result<DriveResult>;
    fn drive_by_angle(&self, angle: f64, speed: f64) -> Result<DriveResult>;
    fn led(&self, r: bool, g: bool, b: bool) -> Result<()>;
    fn servo(&self, degree: f64) -> Result<()>;
    fn buzzer(&self, pw: f64) -> Result<()>;
    fn track_sensor(&self) -> Result<[bool; 4]>;
    fn ultra_sensor(&self) -> Result<f64>;

    fn led_color(&self, color: LedColor) -> Result<()> {
        let (r, g, b) = color.into();
        self.led(r, g, b)
    }

    fn stop(&self) -> Result<()> {
        self.drive(0., 0.)?;
        Ok(())
    }

    fn cleanup(&self) -> Result<()> {
        self.drive(0., 0.)?;
        self.led(false, false, false)?;
        self.servo(0.)?;
        self.buzzer(100.0)?;

        Ok(())
    }
}

pub enum LedColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
}
impl From<LedColor> for (bool, bool, bool) {
    fn from(value: LedColor) -> Self {
        match value {
            // 000
            LedColor::Black => (false, false, false),
            // 001
            LedColor::Red => (true, false, false),
            // 010
            LedColor::Green => (false, true, false),
            // 011
            LedColor::Yellow => (true, true, false),
            // 100
            LedColor::Blue => (false, false, true),
            // 101
            LedColor::Magenta => (true, false, true),
            // 110
            LedColor::Cyan => (false, true, true),
            // 111
            LedColor::White => (true, true, true),
        }
    }
}
