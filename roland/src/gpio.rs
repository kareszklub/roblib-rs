use crate::{constants::*, util::clamp};
pub use anyhow::Error;
use ctrlc::set_handler;
use rppal::{
    gpio::{Gpio, OutputPin},
    pwm::{Channel, Polarity, Pwm},
};
use std::{sync::Mutex, time::Duration};

pub type Result<T> = std::result::Result<T, Error>;

lazy_static::lazy_static! {
    pub static ref GPIO: Gpio = Gpio::new().unwrap();

    pub static ref PIN_R: Mutex<OutputPin> = Mutex::new(GPIO.get(LED_R).unwrap().into_output());
    pub static ref PIN_G: Mutex<OutputPin> = Mutex::new(GPIO.get(LED_G).unwrap().into_output());
    pub static ref PIN_B: Mutex<OutputPin> = Mutex::new(GPIO.get(LED_B).unwrap().into_output());

    pub static ref PIN_SERVO: Mutex<OutputPin> = Mutex::new(GPIO.get(SERVO).unwrap().into_output());

    pub static ref PIN_BUZZER: Mutex<OutputPin> = Mutex::new(GPIO.get(BUZZER).unwrap().into_output());

    pub static ref PIN_FWD_L: Mutex<OutputPin> = Mutex::new(GPIO.get(FWD_L).unwrap().into_output());
    pub static ref PIN_BWD_L: Mutex<OutputPin> = Mutex::new(GPIO.get(BWD_L).unwrap().into_output());
    pub static ref PIN_FWD_R: Mutex<OutputPin> = Mutex::new(GPIO.get(FWD_R).unwrap().into_output());
    pub static ref PIN_BWD_R: Mutex<OutputPin> = Mutex::new(GPIO.get(BWD_R).unwrap().into_output());
}

// TODO: make this actually *try*, currently it just panics
pub fn try_init() -> Result<()> {
    // will attempt to initialize all the pins just to see if they work
    let gpio = Gpio::new()?;

    // LED
    gpio.get(LED_R)?.into_output_low();
    gpio.get(LED_G)?.into_output_low();
    gpio.get(LED_B)?.into_output_low();

    // SERVO
    gpio.get(SERVO)?.into_output_low();

    // BUZZER
    gpio.get(BUZZER)?.into_output_high();

    set_handler(move || {
        eprintln!("Shutting down");
        cleanup().expect("cleanup failed");
    })
    .expect("set_handler failed");

    Ok(())
}

pub fn cleanup() -> Result<()> {
    led(false, false, false)?;
    buzzer(100.0)?;

    Ok(())
}

pub fn led(r: bool, g: bool, b: bool) -> Result<()> {
    let mut pin_r = PIN_R.lock().unwrap();
    let mut pin_g = PIN_G.lock().unwrap();
    let mut pin_b = PIN_B.lock().unwrap();

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

    Ok(())
}

pub fn servo(degree: i8) -> Result<()> {
    let mut pin = PIN_SERVO.lock().unwrap();

    let degree = ((clamp(degree, -90, 90) as i64 + 90) as u64 * 11) + 500;
    pin.set_pwm(Duration::from_millis(20), Duration::from_micros(degree))?;

    dbg!(degree);

    Ok(())
}

pub fn buzzer(pw: f64) -> Result<()> {
    let mut pin = PIN_BUZZER.lock().unwrap();

    if pw == 100.0 {
        pin.clear_pwm()?;
        pin.set_high();
    } else {
        pin.set_pwm_frequency(100.0, pw / 100.0)?;
    }

    Ok(())
}

// TODO
// pub fn drive(left: i8, right: i8) -> Result<()> {
//     let mut pin_fwd_l = PIN_FWD_L.lock().unwrap();
//     let mut pin_bwd_l = PIN_BWD_L.lock().unwrap();
//     let mut pin_fwd_r = PIN_FWD_R.lock().unwrap();
//     let mut pin_bwd_r = PIN_BWD_R.lock().unwrap();

//     // let pwm = Pwm::with_frequency(Channel::Pwm0, 2000.0, 0.0, Polarity::Normal, true);
//     let pwm0 = Pwm::new(Channel::Pwm0);
//     let pwm1 = Pwm::new(Channel::Pwm1);

//     Ok(())
// }
