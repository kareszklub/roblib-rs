#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

use napi::bindgen_prelude::*;
use roblib_client::{
    roblib::{camloc::Camloc, gpio::Gpio, roland::Roland, RoblibBuiltin},
    transports::udp::Udp,
    Robot,
};

#[napi]
pub fn sum(a: i32, b: i32) -> i32 {
    a + b
}

// get real
#[napi]
pub fn sleep(ms: u32) {
    std::thread::sleep(std::time::Duration::from_millis(ms as u64))
}

#[napi(custom_finalize, js_name = "Robot")]
pub struct JsRobot {
    robot: Robot<Udp>,
    active: bool,
}

#[napi]
impl ObjectFinalize for JsRobot {
    // idk xd
    fn finalize(self, _: Env) -> napi::Result<()> {
        // if the connection has been disconnected, then we're good
        if !self.active {
            return Ok(());
        }

        eprintln!("WARN: Robot was dropped with an active connection, refusing to leave...");
        loop {
            std::thread::park();
            eprintln!("parkington");
        }
    }
}

#[napi]
impl JsRobot {
    #[napi(constructor)]
    pub fn new(addr: String) -> Self {
        Self {
            robot: Robot::new(Udp::connect(addr).unwrap()),
            active: true,
        }
    }

    // roblib::RoblibBuiltin
    #[napi]
    pub fn nop(&self) -> anyhow::Result<()> {
        self.robot.nop()
    }

    #[napi]
    pub fn get_uptime(&self) -> anyhow::Result<u32> {
        Ok(self.robot.get_uptime()?.as_secs() as u32)
    }

    // roblib::roland::Roland
    #[napi]
    pub fn drive(&self, left: f64, right: f64) -> anyhow::Result<()> {
        self.robot.drive(left, right)
    }

    #[napi]
    pub fn led(&self, r: bool, g: bool, b: bool) -> anyhow::Result<()> {
        self.robot.led(r, g, b)
    }

    #[napi]
    pub fn roland_servo(&self, degree: f64) -> anyhow::Result<()> {
        roblib_client::roblib::roland::Roland::roland_servo(&self.robot, degree)
    }

    #[napi]
    pub fn buzzer(&self, pw: f64) -> anyhow::Result<()> {
        self.robot.buzzer(pw)
    }

    #[napi]
    pub fn track_sensor(&self) -> anyhow::Result<[bool; 4]> {
        self.robot.track_sensor()
    }

    #[napi]
    pub fn ultra_sensor(&self) -> anyhow::Result<f64> {
        self.robot.ultra_sensor()
    }

    // roblib::gpio::Gpio
    #[napi]
    pub fn read_pin(&self, pin: u8) -> anyhow::Result<bool> {
        self.robot.read_pin(pin)
    }

    #[napi]
    pub fn write_pin(&self, pin: u8, value: bool) -> anyhow::Result<()> {
        self.robot.write_pin(pin, value)
    }

    #[napi]
    pub fn pwm(&self, pin: u8, hz: f64, cycle: f64) -> anyhow::Result<()> {
        self.robot.pwm(pin, hz, cycle)
    }

    #[napi]
    pub fn servo(&self, pin: u8, degree: f64) -> anyhow::Result<()> {
        self.robot.roland_servo(degree)
    }

    #[napi]
    pub fn pin_mode(&self, pin: u8, mode: JsPinMode) -> anyhow::Result<()> {
        self.robot.pin_mode(pin, mode.into())
    }

    // roblib::camloc::Camloc
    #[napi]
    pub fn get_position(&self) -> anyhow::Result<Option<JsPosition>> {
        Ok(self.robot.get_position()?.map(Into::into))
    }
}

#[napi(string_enum, js_name = "PinMode")]
#[allow(non_camel_case_types)]
pub enum JsPinMode {
    input,
    output,
}
impl From<JsPinMode> for roblib_client::roblib::gpio::Mode {
    fn from(value: JsPinMode) -> Self {
        match value {
            JsPinMode::input => roblib_client::roblib::gpio::Mode::Input,
            JsPinMode::output => roblib_client::roblib::gpio::Mode::Output,
        }
    }
}

#[napi(object, js_name = "Position")]
pub struct JsPosition {
    pub x: f64,
    pub y: f64,
    pub rotation: f64,
}
impl From<roblib_client::roblib::camloc::Position> for JsPosition {
    fn from(v: roblib_client::roblib::camloc::Position) -> Self {
        Self {
            x: v.x,
            y: v.y,
            rotation: v.rotation,
        }
    }
}

#[napi]
pub enum EventType {
    GpioPin,

    CamlocConnect,
    CamlocDisconnect,
    CamlocPosition,
    CamlocInfoUpdate,

    None,
}
