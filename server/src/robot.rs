// TODO

use roblib_shared::cmd::SensorData;
use rppal::gpio::{Error as GpioError, Gpio};

//* === PIN NUMBERS ===
// motors
const FWD_L: u8 = 20; // left forward
const BWD_L: u8 = 21; // left backward
const FWD_R: u8 = 19; // right forward
const BWD_R: u8 = 26; // right backward
const PWM_L: u8 = 16; // left speed (pwm)
const PWM_R: u8 = 13; // right speed (pwm)

// led
const LED_R: u8 = 22;
const LED_G: u8 = 27;
const LED_B: u8 = 24;

// infrared sensor pins (bcm)
const TRACK_SENSOR_L1: u8 = 3;
const TRACK_SENSOR_L2: u8 = 5;
const TRACK_SENSOR_R1: u8 = 4;
const TRACK_SENSOR_R2: u8 = 18;

// servo motor
const SERVO: u8 = 4;

// buzzer
const BUZZER: u8 = 10;

// ultrasonic
const ECHO: u8 = 0;
const TRIG: u8 = 1;

/// A trait specifying all the functions of the robot.
pub trait RobotTrait {
    fn led(&self, r: bool, g: bool, b: bool);
    fn move_robot(&self, left: i8, right: i8);
    fn stop_robot(&self);
    fn servo_absolute(&self, degree: f32);
    fn track_sensor(&self) -> SensorData;
    fn buzzer(&self, pw: f32);

    fn box_clone(&self) -> Robot;
}
impl Clone for Robot {
    fn clone(&self) -> Robot {
        self.box_clone()
    }
}
pub type Robot = Box<dyn RobotTrait + Sync + Send>;

/// This implementation is used when running on the actual robot.
#[derive(Clone)]
pub struct RealRobot {
    gpio: Gpio,
}
impl RealRobot {
    /// attempts to initialize the robot, will return none if anyhting fails
    pub fn new() -> Result<RealRobot, GpioError> {
        let gpio = Gpio::new()?;

        // will attempt to initialize all the pins just to see if they work
        gpio.get(LED_R)?;
        gpio.get(LED_G)?;
        gpio.get(LED_B)?;

        Ok(RealRobot { gpio })
    }
}
impl RobotTrait for RealRobot {
    fn led(&self, r: bool, g: bool, b: bool) {
        info!("LED: {}:{}:{}", r, g, b);

        let mut pin_r = self
            .gpio
            .get(LED_R)
            .expect("falied to get pin_r")
            .into_output();
        let mut pin_g = self
            .gpio
            .get(LED_G)
            .expect("falied to get pin_g")
            .into_output();
        let mut pin_b = self
            .gpio
            .get(LED_B)
            .expect("falied to get pin_b")
            .into_output();

        // don't reset the pins when these variables go out of scope
        pin_r.set_reset_on_drop(false);
        pin_g.set_reset_on_drop(false);
        pin_b.set_reset_on_drop(false);

        if r {
            pin_r.set_high();
        } else {
            pin_r.set_low();
        }
        if g {
            pin_g.set_high();
        } else {
            pin_g.set_low();
        }
        if b {
            pin_b.set_high();
        } else {
            pin_b.set_low();
        }
    }

    // TODO: implement
    fn move_robot(&self, left: i8, right: i8) {
        info!("Moving robot: {}:{}", left, right);
    }
    fn stop_robot(&self) {
        info!("Stopping robot");
    }
    fn servo_absolute(&self, degree: f32) {
        info!("Servo absolute: {}", degree);
    }
    fn track_sensor(&self) -> SensorData {
        info!("Track sensor");
        [0, 1, 2, 3]
    }
    fn buzzer(&self, pw: f32) {
        info!("Buzzer: {}", pw);
    }

    fn box_clone(&self) -> Robot {
        Box::new(self.clone())
    }
}

/// This implementation is used to simpulate the robot.
/// This allows the program to run without calling any IO code and causing it to crash on a normal pc.
#[derive(Clone)]
pub struct MockRobot {}
impl MockRobot {
    pub fn new() -> MockRobot {
        MockRobot {}
    }
}
impl RobotTrait for MockRobot {
    fn led(&self, r: bool, g: bool, b: bool) {
        info!("LED: {}:{}:{}", r, g, b);
    }
    fn move_robot(&self, left: i8, right: i8) {
        info!("Moving robot: {}:{}", left, right);
    }
    fn stop_robot(&self) {
        info!("Stopping robot");
    }
    fn servo_absolute(&self, degree: f32) {
        info!("Servo absolute: {}", degree);
    }
    fn track_sensor(&self) -> SensorData {
        info!("Track sensor");
        [0, 1, 2, 3]
    }
    fn buzzer(&self, pw: f32) {
        info!("Buzzer: {}", pw);
    }

    fn box_clone(&self) -> Robot {
        Box::new(self.clone())
    }
}

/// this function should be called when wanting to interact with the robot,
/// as it ensures the proper variant is used.
pub fn init_robot() -> (Robot, Option<GpioError>) {
    match RealRobot::new() {
        Ok(robot) => (Box::new(robot), None),
        Err(e) => (Box::new(MockRobot::new()), Some(e)),
    }
}
