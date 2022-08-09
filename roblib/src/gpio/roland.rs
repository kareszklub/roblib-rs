use crate::gpio::{clamp, constants::*, Result};
pub mod constants {
    pub use crate::gpio::constants::*;
}

use ctrlc::set_handler;
use rppal::gpio::{Gpio, InputPin, OutputPin};
use std::{cmp::Ordering, sync::Mutex, time::Duration};

lazy_static::lazy_static! {
    static ref GPIO: Gpio = Gpio::new().unwrap();
    pub static ref RES: Option<anyhow::Error> = try_init().err();

    static ref PIN_R: Mutex<OutputPin> = Mutex::new(GPIO.get(LED_R).unwrap().into_output());
    static ref PIN_G: Mutex<OutputPin> = Mutex::new(GPIO.get(LED_G).unwrap().into_output());
    static ref PIN_B: Mutex<OutputPin> = Mutex::new(GPIO.get(LED_B).unwrap().into_output());

    static ref PIN_SERVO: Mutex<OutputPin> = Mutex::new(GPIO.get(SERVO).unwrap().into_output());

    static ref PIN_BUZZER: Mutex<OutputPin> = Mutex::new(GPIO.get(BUZZER).unwrap().into_output());

    static ref PIN_FWD_L: Mutex<OutputPin> = Mutex::new(GPIO.get(FWD_L).unwrap().into_output());
    static ref PIN_BWD_L: Mutex<OutputPin> = Mutex::new(GPIO.get(BWD_L).unwrap().into_output());
    static ref PIN_PWM_L: Mutex<OutputPin> = Mutex::new(GPIO.get(PWM_L).unwrap().into_output());
    static ref PIN_FWD_R: Mutex<OutputPin> = Mutex::new(GPIO.get(FWD_R).unwrap().into_output());
    static ref PIN_BWD_R: Mutex<OutputPin> = Mutex::new(GPIO.get(BWD_R).unwrap().into_output());
    static ref PIN_PWM_R: Mutex<OutputPin> = Mutex::new(GPIO.get(PWM_R).unwrap().into_output());

    static ref PIN_TRACK_L1: Mutex<InputPin> = Mutex::new(GPIO.get(TRACK_L1).unwrap().into_input());
    static ref PIN_TRACK_L2: Mutex<InputPin> = Mutex::new(GPIO.get(TRACK_L2).unwrap().into_input());
    static ref PIN_TRACK_R1: Mutex<InputPin> = Mutex::new(GPIO.get(TRACK_R1).unwrap().into_input());
    static ref PIN_TRACK_R2: Mutex<InputPin> = Mutex::new(GPIO.get(TRACK_R2).unwrap().into_input());
}

pub fn try_init() -> Result<()> {
    // will attempt to initialize all the pins just to see if they work
    let gpio = Gpio::new()?;

    // MOTOR
    gpio.get(FWD_L)?.into_output_low();
    gpio.get(BWD_L)?.into_output_low();
    gpio.get(PWM_L)?.into_output_high();
    gpio.get(FWD_R)?.into_output_low();
    gpio.get(BWD_R)?.into_output_low();
    gpio.get(PWM_R)?.into_output_high();

    // LED
    gpio.get(LED_R)?.into_output_low();
    gpio.get(LED_G)?.into_output_low();
    gpio.get(LED_B)?.into_output_low();

    // SERVO
    gpio.get(SERVO)?.into_output_high();

    // BUZZER
    gpio.get(BUZZER)?.into_output_high();

    set_handler(move || {
        eprintln!("Shutting down");
        cleanup().expect("cleanup failed");
    })
    .expect("set_handler failed");

    // ran here as well to reset servo to center
    cleanup()?;

    Ok(())
}

pub fn cleanup() -> Result<()> {
    drive(0, 0)?;
    led(false, false, false)?;
    servo(0)?;
    buzzer(100.0)?;

    Ok(())
}

pub fn drive(left: i8, right: i8) -> Result<()> {
    let mut pin_fwd_l = PIN_FWD_L.lock().unwrap();
    let mut pin_bwd_l = PIN_BWD_L.lock().unwrap();
    let mut pin_pwm_l = PIN_PWM_L.lock().unwrap();
    let mut pin_fwd_r = PIN_FWD_R.lock().unwrap();
    let mut pin_bwd_r = PIN_BWD_R.lock().unwrap();
    let mut pin_pwm_r = PIN_PWM_R.lock().unwrap();

    pin_pwm_l.set_pwm_frequency(2000.0, left.abs() as f64 / 100.0)?;
    pin_pwm_r.set_pwm_frequency(2000.0, right.abs() as f64 / 100.0)?;

    match left.cmp(&0) {
        Ordering::Greater => {
            pin_fwd_l.set_high();
            pin_bwd_l.set_low();
        }
        Ordering::Less => {
            pin_fwd_l.set_low();
            pin_bwd_l.set_high();
        }
        Ordering::Equal => {
            pin_fwd_l.set_low();
            pin_bwd_l.set_low();
        }
    }
    match right.cmp(&0) {
        Ordering::Greater => {
            pin_fwd_r.set_high();
            pin_bwd_r.set_low();
        }
        Ordering::Less => {
            pin_fwd_r.set_low();
            pin_bwd_r.set_high();
        }
        Ordering::Equal => {
            pin_fwd_r.set_low();
            pin_bwd_r.set_low();
        }
    }

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
    pin.set_pwm(Duration::from_millis(20), Duration::from_micros(degree))?; // 50Hz

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

pub fn track_sensor() -> Result<[bool; 4]> {
    let pin_l1 = PIN_TRACK_L1.lock().unwrap();
    let pin_l2 = PIN_TRACK_L2.lock().unwrap();
    let pin_r1 = PIN_TRACK_R1.lock().unwrap();
    let pin_r2 = PIN_TRACK_R2.lock().unwrap();

    Ok([
        pin_l1.is_high(),
        pin_l2.is_high(),
        pin_r1.is_high(),
        pin_r2.is_high(),
    ])
}
