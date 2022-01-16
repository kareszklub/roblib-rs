use roblib_shared::cmd::SensorData;
use rppal::{gpio::Gpio, system::DeviceInfo};

lazy_static! {
    static ref IS_ROBOT: bool = DeviceInfo::new().is_ok();
}

const LED_R: u8 = 22;
const LED_G: u8 = 27;
const LED_B: u8 = 24;

macro_rules! check_robot {
    () => {
        if *IS_ROBOT {
            info!("running on robot");
        } else {
            info!("not running on robot");
            return;
        }
    };
}

pub fn move_robot(left: i8, right: i8) {
    info!("Moving robot: {}:{}", left, right);
    check_robot!();
}

pub fn stop_robot() {
    info!("Stopping robot");
    check_robot!();

    let dev = DeviceInfo::new().unwrap();
    info!("{}", dev.model());
}

pub fn led(r: bool, g: bool, b: bool) {
    info!("LED: {}:{}:{}", r, g, b);
    check_robot!();

    // let gpio = Gpio::new().unwrap();
    // let mut pin_r = gpio.get(LED_R).unwrap().into_output();
    // let mut pin_g = gpio.get(LED_G).unwrap().into_output();
    // let mut pin_b = gpio.get(LED_B).unwrap().into_output();
    // if r {
    //     pin_r.set_high();
    // } else {
    //     pin_r.set_low();
    // }
    // if g {
    //     pin_g.set_high();
    // } else {
    //     pin_g.set_low();
    // }
    // if b {
    //     pin_b.set_high();
    // } else {
    //     pin_b.set_low();
    // }

    let mut pin_r = Gpio::new().unwrap().get(LED_R).unwrap().into_output();
    pin_r.set_high();
    std::thread::sleep(std::time::Duration::from_secs(3));
    pin_r.set_low();
}

pub fn servo_absolute(degree: f32) {
    info!("Servo absolute: {}", degree);
    check_robot!();
}

pub fn track_sensor() -> SensorData {
    info!("Track sensor");
    if *IS_ROBOT {
        info!("running on robot");
    } else {
        info!("not running on robot");
        return [0, 0, 0, 0];
    }
    [0, 1, 2, 3]
}

pub fn buzzer(pw: f32) {
    info!("Buzzer: {}", pw);
    check_robot!();
}
