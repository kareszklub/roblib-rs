#[macro_use]
extern crate napi_derive;

use napi::{
    bindgen_prelude::*,
    threadsafe_function::{ErrorStrategy, ThreadSafeCallContext, ThreadsafeFunction},
    tokio::{self, sync::broadcast::error::RecvError},
};
use roblib_client::{
    roblib::{
        camloc::{CamlocAsync, Position},
        event::{self, ConcreteValue},
        gpio::{GpioAsync, Mode},
        roland::RolandAsync,
        RoblibBuiltinAsync,
    },
    transports::tcp::TcpAsync,
    RobotAsync,
};
use std::{sync::Arc, time::Duration};

macro_rules! sub_recv {
    ($robot:ident, $tsfn:ident, $ev:expr) => {
        let mut rx = $robot.subscribe($ev).await?;
        sub_loop!(rx, $tsfn);
    };
    ($robot:ident, $tsfn:ident, $ev:expr, $ev_args:ident) => {
        let e = anyhow::anyhow!("Invalid event args: {:?}", &$ev_args);
        let Ok(v) = serde_json::from_value($ev_args) else {
                                                                                    return Err(e);
                                                                                };
        let mut rx = $robot.subscribe($ev(v)).await?;
        sub_loop!(rx, $tsfn);
    };
}
macro_rules! sub_loop {
    ($rx:ident, $tsfn:ident) => {
        loop {
            let msg = match $rx.recv().await {
                Ok(msg) => msg,
                Err(RecvError::Lagged(n)) => {
                    eprintln!("event handler skipped {n} messages");
                    continue;
                }
                Err(RecvError::Closed) => {
                    panic!("channel closed");
                }
            };
            $tsfn
                .call_async(Ok(vec![serde_json::to_value(msg)?]))
                .await?;
        }
    };
}

#[napi]
pub fn sum(a: i32, b: i32) -> i32 {
    a + b
}

// get real
#[napi]
pub fn sleep_blocking(ms: u32) {
    std::thread::sleep(Duration::from_millis(ms as u64))
}

#[napi]
pub async fn sleep(ms: u32) {
    tokio::time::sleep(Duration::from_millis(ms as u64)).await
}

#[napi]
pub async fn throws(n: u32) -> Result<()> {
    Err(anyhow::anyhow!("ERROR: {n}"))?
}

#[napi(custom_finalize, js_name = "Robot")]
#[derive(Clone)]
pub struct JsRobot {
    robot: Arc<RobotAsync<TcpAsync>>,
    active: bool,
    rt: Arc<tokio::runtime::Runtime>,
}

#[napi]
impl ObjectFinalize for JsRobot {
    fn finalize(self, _: Env) -> napi::Result<()> {
        if self.active {
            log::warn!("Robot was dropped with an active connection");
        }
        // if the connection has been disconnected, then we're good
        return Ok(());
    }
}

#[napi]
impl JsRobot {
    #[napi(constructor)]
    pub fn new() -> Self {
        panic!("Use Robot.connect instead of new Robot")
    }

    #[napi]
    pub async fn connect(addr: String) -> Result<JsRobot> {
        roblib_client::logger::init_log(Some("info"));
        let rt = Arc::new(
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .thread_name("roblib-rt")
                .build()?,
        );

        let tcp = rt.spawn(TcpAsync::connect(addr)).await.unwrap().unwrap();

        Ok(Self {
            robot: Arc::new(RobotAsync::new(tcp)),
            active: true,
            rt,
        })
    }

    #[napi]
    // get real
    pub unsafe fn disconnect(&mut self) -> Result<()> {
        self.active = false;
        let x: *const tokio::runtime::Runtime = &*self.rt;
        let rt = std::ptr::read(x);
        rt.shutdown_timeout(Duration::from_millis(100));
        Ok(())
    }

    // roblib::RoblibBuiltin
    #[napi]
    pub async fn nop(&self) -> Result<()> {
        Ok(self.robot.nop().await?)
    }

    #[napi]
    pub async fn get_uptime(&self) -> Result<u32> {
        Ok(self.robot.get_uptime().await?.as_secs() as u32)
    }

    // roblib::roland::Roland
    #[napi]
    pub async fn drive(&self, left: f64, right: f64) -> Result<()> {
        Ok(self.robot.drive(left, right).await?)
    }

    #[napi]
    pub async fn led(&self, r: bool, g: bool, b: bool) -> Result<()> {
        Ok(self.robot.led(r, g, b).await?)
    }

    #[napi]
    pub async fn roland_servo(&self, degree: f64) -> Result<()> {
        Ok(self.robot.roland_servo(degree).await?)
    }

    #[napi]
    pub async fn buzzer(&self, pw: f64) -> Result<()> {
        Ok(self.robot.buzzer(pw).await?)
    }

    #[napi]
    pub async fn track_sensor(&self) -> Result<[bool; 4]> {
        Ok(self.robot.track_sensor().await?)
    }

    #[napi]
    pub async fn ultra_sensor(&self) -> Result<f64> {
        Ok(self.robot.ultra_sensor().await?)
    }

    // roblib::gpio::Gpio
    #[napi]
    pub async fn read_pin(&self, pin: u8) -> Result<bool> {
        Ok(self.robot.read_pin(pin).await?)
    }

    #[napi]
    pub async fn write_pin(&self, pin: u8, value: bool) -> Result<()> {
        Ok(self.robot.write_pin(pin, value).await?)
    }

    #[napi]
    pub async fn pwm(&self, pin: u8, hz: f64, cycle: f64) -> Result<()> {
        Ok(self.robot.pwm(pin, hz, cycle).await?)
    }

    #[napi]
    pub async fn servo(&self, pin: u8, degree: f64) -> Result<()> {
        Ok(self.robot.servo(pin, degree).await?)
    }

    #[napi]
    pub async fn pin_mode(&self, pin: u8, mode: JsPinMode) -> Result<()> {
        Ok(self.robot.pin_mode(pin, mode.into()).await?)
    }

    // roblib::camloc::Camloc
    #[napi]
    pub async fn get_position(&self) -> Result<Option<JsPosition>> {
        Ok(self.robot.get_position().await?.map(Into::into))
    }

    // events
    #[napi]
    pub fn subscribe(
        &self,
        ev: JsEventType,
        ev_args: serde_json::Value,
        handler: JsFunction,
    ) -> Result<()> {
        let tsfn: ThreadsafeFunction<Vec<serde_json::Value>, ErrorStrategy::CalleeHandled> =
            handler.create_threadsafe_function(
                0,
                |ctx: ThreadSafeCallContext<Vec<serde_json::Value>>| Ok(ctx.value),
            )?;

        let robot = self.robot.clone();
        self.rt.spawn(async move {
            match ev {
                JsEventType::TrackSensor => {
                    sub_recv!(robot, tsfn, event::TrackSensor);
                }
                JsEventType::UltraSensor => {
                    sub_recv!(robot, tsfn, event::UltraSensor, ev_args);
                }
                JsEventType::GpioPin => {
                    sub_recv!(robot, tsfn, event::GpioPin, ev_args);
                }
                JsEventType::CamlocConnect => {
                    sub_recv!(robot, tsfn, event::CamlocConnect);
                }
                JsEventType::CamlocDisconnect => {
                    sub_recv!(robot, tsfn, event::CamlocDisconnect);
                }
                JsEventType::CamlocPosition => {
                    sub_recv!(robot, tsfn, event::CamlocPosition);
                }
                JsEventType::CamlocInfoUpdate => {
                    sub_recv!(robot, tsfn, event::CamlocInfoUpdate);
                }
            }
            // force return type
            #[allow(unreachable_code)]
            anyhow::Ok(())
        });
        Ok(())
    }

    // async fn unsubscribe(&self, ev: E) -> Result<()> { }
}

#[napi(string_enum, js_name = "EventType")]
pub enum JsEventType {
    TrackSensor,
    UltraSensor,

    GpioPin,

    CamlocConnect,
    CamlocDisconnect,
    CamlocPosition,
    CamlocInfoUpdate,
}
impl JsEventType {
    pub fn to_concrete(self, value: serde_json::Value) {
        match self {
            JsEventType::TrackSensor => ConcreteValue::TrackSensor(
                serde_json::from_value(value).expect("invalid event value"),
            ),
            JsEventType::UltraSensor => ConcreteValue::UltraSensor(
                serde_json::from_value(value).expect("invalid event value"),
            ),
            JsEventType::GpioPin => {
                ConcreteValue::GpioPin(serde_json::from_value(value).expect("invalid event value"))
            }
            JsEventType::CamlocConnect => ConcreteValue::CamlocConnect(
                serde_json::from_value(value).expect("invalid event value"),
            ),
            JsEventType::CamlocDisconnect => ConcreteValue::CamlocDisconnect(
                serde_json::from_value(value).expect("invalid event value"),
            ),
            JsEventType::CamlocPosition => ConcreteValue::CamlocPosition(
                serde_json::from_value(value).expect("invalid event value"),
            ),
            JsEventType::CamlocInfoUpdate => ConcreteValue::CamlocInfoUpdate(
                serde_json::from_value(value).expect("invalid event value"),
            ),
        };
    }
}

#[napi(string_enum, js_name = "PinMode")]
#[allow(non_camel_case_types)]
pub enum JsPinMode {
    input,
    output,
}
impl From<JsPinMode> for Mode {
    fn from(value: JsPinMode) -> Self {
        match value {
            JsPinMode::input => Mode::Input,
            JsPinMode::output => Mode::Output,
        }
    }
}

#[napi(object, js_name = "Position")]
pub struct JsPosition {
    pub x: f64,
    pub y: f64,
    pub rotation: f64,
}
impl From<Position> for JsPosition {
    fn from(v: Position) -> Self {
        Self {
            x: v.x,
            y: v.y,
            rotation: v.rotation,
        }
    }
}
