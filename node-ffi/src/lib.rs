#[macro_use]
extern crate napi_derive;

use napi::{
    bindgen_prelude::*,
    threadsafe_function::{
        ErrorStrategy, ThreadSafeCallContext, ThreadsafeFunction, ThreadsafeFunctionCallMode,
    },
    tokio::{self, sync::broadcast::error::RecvError},
};
use roblib_client::{
    roblib::{
        camloc::{CamlocAsync, Position},
        event,
        gpio::{GpioAsync, Mode},
        roland::RolandAsync,
        RoblibBuiltinAsync,
    },
    transports::tcp::TcpAsync,
    RobotAsync,
};
use std::{sync::Arc, time::Duration};

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
            eprintln!("WARN: Robot was dropped with an active connection, refusing to leave...");
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
        roblib_client::logger::init_log(None);
        let rt = Arc::new(
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .thread_name("roblib-rt")
                .build()?,
        );
        let _ = rt.spawn(async {
            dbg!();
            for i in 0.. {
                dbg!(i);
                sleep(250).await
            }
        });

        Ok(Self {
            robot: Arc::new(RobotAsync::new(TcpAsync::connect(addr).await?)),
            active: true,
            rt,
        })
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
        value: serde_json::Value,
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
                    let mut rx = robot.subscribe(event::TrackSensor).await?;
                    loop {
                        let msg = match rx.recv().await {
                            Ok(msg) => msg,
                            Err(RecvError::Lagged(n)) => {
                                eprintln!("event handler skipped {n} messages");
                                continue;
                            }
                            Err(RecvError::Closed) => {
                                panic!("channel closed");
                            }
                        };
                        tsfn.call_async(Ok(vec![serde_json::to_value(msg)?]))
                            .await?;
                    }
                }
                JsEventType::UltraSensor => {
                    let v = serde_json::from_value(value).expect("invalid event value");
                    let rx = robot.subscribe(event::UltraSensor(v)).await?;
                }
                JsEventType::GpioPin => {
                    let mut rx = robot.subscribe(event::GpioPin(2)).await?;
                    loop {
                        dbg!();
                        let msg = match rx.recv().await {
                            Ok(msg) => msg,
                            Err(RecvError::Lagged(n)) => {
                                eprintln!("event handler skipped {n} messages");
                                continue;
                            }
                            Err(RecvError::Closed) => {
                                panic!("channel closed");
                            }
                        };
                        dbg!();
                        tsfn.call_async(Ok(vec![serde_json::to_value(msg)?]))
                            .await?;
                    }
                }
                JsEventType::CamlocConnect => {
                    let rx = robot.subscribe(event::CamlocConnect).await?;
                }
                JsEventType::CamlocDisconnect => {
                    let rx = robot.subscribe(event::CamlocDisconnect).await?;
                }
                JsEventType::CamlocPosition => {
                    let rx = robot.subscribe(event::CamlocPosition).await?;
                }
                JsEventType::CamlocInfoUpdate => {
                    let rx = robot.subscribe(event::CamlocInfoUpdate).await?;
                }
            }
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
            JsEventType::TrackSensor => {
                event::ConcreteValue::TrackSensor(serde_json::from_value(value).expect("invalid event value"))
            }
            JsEventType::UltraSensor => {
                event::ConcreteValue::UltraSensor(serde_json::from_value(value).expect("invalid event value"))
            }
            JsEventType::GpioPin => {
                event::ConcreteValue::GpioPin(serde_json::from_value(value).expect("invalid event value"))
            }
            JsEventType::CamlocConnect => {
                event::ConcreteValue::CamlocConnect(serde_json::from_value(value).expect("invalid event value"))
            }
            JsEventType::CamlocDisconnect => {
                event::ConcreteValue::CamlocDisconnect(serde_json::from_value(value).expect("invalid event value"))
            }
            JsEventType::CamlocPosition => {
                event::ConcreteValue::CamlocPosition(serde_json::from_value(value).expect("invalid event value"))
            }
            JsEventType::CamlocInfoUpdate => {
                event::ConcreteValue::CamlocInfoUpdate(serde_json::from_value(value).expect("invalid event value"))
            }
            // JsEventType::TrackSensor => event::ConcreteType::TrackSensor(
            //     serde_json::from_value(value).expect("invalid event value"),
            // ),
            // JsEventType::UltraSensor => event::ConcreteType::UltraSensor(
            //     serde_json::from_value(value).expect("invalid event value"),
            // ),
            // JsEventType::GpioPin => event::ConcreteType::GpioPin(
            //     serde_json::from_value(value).expect("invalid event value"),
            // ),
            // JsEventType::CamlocConnect => event::ConcreteType::CamlocConnect(
            //     serde_json::from_value(value).expect("invalid event value"),
            // ),
            // JsEventType::CamlocDisconnect => event::ConcreteType::CamlocDisconnect(
            //     serde_json::from_value(value).expect("invalid event value"),
            // ),
            // JsEventType::CamlocPosition => event::ConcreteType::CamlocPosition(
            //     serde_json::from_value(value).expect("invalid event value"),
            // ),
            // JsEventType::CamlocInfoUpdate => event::ConcreteType::CamlocInfoUpdate(
            //     serde_json::from_value(value).expect("invalid event value"),
            // ),
            // JsEventType::None => event::ConcreteType::None,
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

#[napi]
pub fn test_threadsafe_function(func: JsFunction) -> Result<()> {
    // let func = ctx.get::<JsFunction>(0)?;

    let tsfn: ThreadsafeFunction<Vec<serde_json::Value>, ErrorStrategy::CalleeHandled> = func
        .create_threadsafe_function(0, |ctx: ThreadSafeCallContext<Vec<serde_json::Value>>| {
            Ok(ctx.value)
        })?;

    // let tsfn_cloned = tsfn.clone();

    std::thread::spawn(move || {
        let output: Vec<serde_json::Value> = vec![0, 1, 2, 3].into_iter().map(Into::into).collect();
        // It's okay to call a threadsafe function multiple times.
        tsfn.call(Ok(output.clone()), ThreadsafeFunctionCallMode::Blocking);
    });

    // std::thread::spawn(move || {
    //     let output: Vec<u32> = vec![3, 2, 1, 0];
    //     // It's okay to call a threadsafe function multiple times.
    //     tsfn_cloned.call(Ok(output.clone()), ThreadsafeFunctionCallMode::NonBlocking);
    // });

    Ok(())
}
