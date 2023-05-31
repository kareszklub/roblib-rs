use crate::gpio::{clamp, constants::*, Result};
pub mod constants {
    pub use crate::gpio::constants::*;
}

use camloc_server::{
    compass::{no_compass, Compass},
    extrapolations::{Extrapolation, LinearExtrapolation},
    service::{LocationService, LocationServiceHandle},
    TimedPosition,
};
use rppal::gpio::{Gpio, InputPin, OutputPin};
use std::{cmp::Ordering, sync::Mutex, time::Duration};

pub struct Roland {
    pin_r: Mutex<OutputPin>,
    pin_g: Mutex<OutputPin>,
    pin_b: Mutex<OutputPin>,

    pin_servo: Mutex<OutputPin>,
    pin_fwd_l: Mutex<OutputPin>,
    pin_bwd_l: Mutex<OutputPin>,
    pin_pwm_l: Mutex<OutputPin>,
    pin_fwd_r: Mutex<OutputPin>,
    pin_bwd_r: Mutex<OutputPin>,
    pin_pwm_r: Mutex<OutputPin>,

    pin_buzzer: Mutex<OutputPin>,

    pin_track_l1: Mutex<InputPin>,
    pin_track_l2: Mutex<InputPin>,
    pin_track_r1: Mutex<InputPin>,
    pin_track_r2: Mutex<InputPin>,

    camloc_service: LocationServiceHandle,
}

impl Drop for Roland {
    fn drop(&mut self) {
        self.cleanup().expect("Failed to clean up!!!");
    }
}

impl Roland {
    pub async fn try_init() -> Result<Roland> {
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

        let camloc_service = LocationService::start(
            Some(Extrapolation::new::<LinearExtrapolation>(
                Duration::from_millis(500),
            )),
            // no_extrapolation!(),
            camloc_server::camloc_common::hosts::constants::MAIN_PORT,
            no_compass!(),
            Duration::from_millis(500),
        )
        .await
        .map_err(anyhow::Error::msg)?;

        let roland = Roland {
            pin_r: Mutex::new(gpio.get(LED_R)?.into_output()),
            pin_g: Mutex::new(gpio.get(LED_G)?.into_output()),
            pin_b: Mutex::new(gpio.get(LED_B)?.into_output()),
            pin_servo: Mutex::new(gpio.get(SERVO)?.into_output()),
            pin_buzzer: Mutex::new(gpio.get(BUZZER)?.into_output()),
            pin_fwd_l: Mutex::new(gpio.get(FWD_L)?.into_output()),
            pin_bwd_l: Mutex::new(gpio.get(BWD_L)?.into_output()),
            pin_pwm_l: Mutex::new(gpio.get(PWM_L)?.into_output()),
            pin_fwd_r: Mutex::new(gpio.get(FWD_R)?.into_output()),
            pin_bwd_r: Mutex::new(gpio.get(BWD_R)?.into_output()),
            pin_pwm_r: Mutex::new(gpio.get(PWM_R)?.into_output()),
            pin_track_l1: Mutex::new(gpio.get(TRACK_L1)?.into_input()),
            pin_track_l2: Mutex::new(gpio.get(TRACK_L2)?.into_input()),
            pin_track_r1: Mutex::new(gpio.get(TRACK_R1)?.into_input()),
            pin_track_r2: Mutex::new(gpio.get(TRACK_R2)?.into_input()),

            camloc_service,
        };

        // ran here as well to reset servo to center
        roland.cleanup()?;

        Ok(roland)
    }

    pub fn cleanup(&self) -> Result<()> {
        self.drive(0, 0)?;
        self.led(false, false, false)?;
        self.servo(0)?;
        self.buzzer(100.0)?;

        Ok(())
    }

    pub fn drive(&self, left: i8, right: i8) -> Result<()> {
        let mut pin_fwd_l = self.pin_fwd_l.lock().unwrap();
        let mut pin_bwd_l = self.pin_bwd_l.lock().unwrap();
        let mut pin_pwm_l = self.pin_pwm_l.lock().unwrap();
        let mut pin_fwd_r = self.pin_fwd_r.lock().unwrap();
        let mut pin_bwd_r = self.pin_bwd_r.lock().unwrap();
        let mut pin_pwm_r = self.pin_pwm_r.lock().unwrap();

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

    pub fn drive_by_angle(&self, angle: f64, speed: i8) -> Result<()> {
        let angle = clamp(angle, -90.0, 90.0);
        let speed = clamp(speed, -100, 100);

        let a = (angle + 90.0) / 180.0;

        let speed_left = (a * 100.0) * speed as f64;
        let speed_right = (100.0 - (a * 100.0)) * speed as f64;

        self.drive(speed_left as i8, speed_right as i8)?;

        Ok(())
    }

    pub fn led(&self, r: bool, g: bool, b: bool) -> Result<()> {
        let mut pin_r = self.pin_r.lock().unwrap();
        let mut pin_g = self.pin_g.lock().unwrap();
        let mut pin_b = self.pin_b.lock().unwrap();

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

    pub fn servo(&self, degree: i8) -> Result<()> {
        let mut pin = self.pin_servo.lock().unwrap();

        let degree = ((clamp(degree, -90, 90) as i64 + 90) as u64 * 11) + 500;
        pin.set_pwm(Duration::from_millis(20), Duration::from_micros(degree))?; // 50Hz

        Ok(())
    }

    pub fn buzzer(&self, pw: f64) -> Result<()> {
        let mut pin = self.pin_buzzer.lock().unwrap();

        if pw == 100.0 {
            pin.clear_pwm()?;
            pin.set_high();
        } else {
            pin.set_pwm_frequency(100.0, pw / 100.0)?;
        }

        Ok(())
    }

    pub fn track_sensor(&self) -> Result<[bool; 4]> {
        let pin_l1 = self.pin_track_l1.lock().unwrap();
        let pin_l2 = self.pin_track_l2.lock().unwrap();
        let pin_r1 = self.pin_track_r1.lock().unwrap();
        let pin_r2 = self.pin_track_r2.lock().unwrap();

        Ok([
            pin_l1.is_high(),
            pin_l2.is_high(),
            pin_r1.is_high(),
            pin_r2.is_high(),
        ])
    }

    pub fn get_position(&self) -> Option<TimedPosition> {
        camloc_server::tokio::task::block_in_place(|| {
            camloc_server::tokio::runtime::Handle::current()
                .block_on(self.camloc_service.get_position())
        })
    }
}
