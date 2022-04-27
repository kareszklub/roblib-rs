use roblib_shared::cmd::SensorData;
use roland::{constants::*, Error as GpioError, Gpio};

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

pub type Robot = Box<dyn RobotTrait + Sync + Send>;
impl Clone for Robot {
    fn clone(&self) -> Robot {
        self.box_clone()
    }
}

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

        roland::led(&self.gpio, r, g, b).expect("failed to initialize led pins")
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