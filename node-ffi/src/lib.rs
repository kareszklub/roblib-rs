#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

use napi::{bindgen_prelude::ObjectFinalize, Env};
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
    fn finalize(self, mut env: Env) -> napi::Result<()> {
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

    // RoblibBuiltin
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
        roblib_client::roblib::roland::Roland::servo(&self.robot, degree)
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
    pub fn set_pin(&self, pin: u8, value: bool) -> anyhow::Result<()> {
        self.robot.set_pin(pin, value)
    }

    #[napi]
    pub fn pwm(&self, pin: u8, hz: f64, cycle: f64) -> anyhow::Result<()> {
        self.robot.pwm(pin, hz, cycle)
    }

    #[napi]
    pub fn servo(&self, pin: u8, degree: f64) -> anyhow::Result<()> {
        roblib_client::roblib::gpio::Gpio::servo(&self.robot, pin, degree)
    }

    // roblib::camloc::Camloc
    #[napi]
    pub fn get_position(&self) -> anyhow::Result<Option<roblib_client::roblib::camloc::Position>> {
        self.robot.get_position()
    }
}
