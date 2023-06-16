pub mod cmd;

#[cfg(feature = "gpio-backend")]
pub mod backend;

pub trait Roland: Sized {
    fn drive(&self, left: f64, right: f64) -> anyhow::Result<()>;
    fn drive_by_angle(&self, angle: f64, speed: f64) -> anyhow::Result<()>;
    fn led(&self, r: bool, g: bool, b: bool) -> anyhow::Result<()>;
    fn servo(&self, degree: f64) -> anyhow::Result<()>;
    fn buzzer(&self, pw: f64) -> anyhow::Result<()>;
    fn track_sensor(&self) -> anyhow::Result<[bool; 4]>;
    fn ultra_sensor(&self) -> anyhow::Result<f64>;

    fn led_color(&self, color: LedColor) -> anyhow::Result<()> {
        let (r, g, b) = color.into();
        self.led(r, g, b)
    }

    fn stop(&self) -> anyhow::Result<()> {
        self.drive(0., 0.)?;
        Ok(())
    }

    fn cleanup(&self) -> anyhow::Result<()> {
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

pub fn convert_move(angle: f64, speed: f64) -> (f64, f64) {
    let angle = angle.clamp(-90.0, 90.0);
    let speed = speed.clamp(-1., 1.);

    let a = (angle + 90.0) / 180.0;

    let left = (a * 100.0) * speed;
    let right = (100.0 - (a * 100.0)) * speed;

    (left, right)
}
