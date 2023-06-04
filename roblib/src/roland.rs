#[cfg(feature = "camloc")]
use camloc_server::MotionHint;
use rppal::gpio::{Gpio, InputPin, OutputPin};
use std::{
    sync::Mutex,
    time::{Duration, Instant},
};

use crate::{clamp, get_servo_pwm_durations};
use anyhow::Result;
use constants::*;

pub mod constants {
    // motors
    pub const FWD_L: u8 = 20; // left forward
    pub const BWD_L: u8 = 21; // left backward
    pub const FWD_R: u8 = 19; // right forward
    pub const BWD_R: u8 = 26; // right backward
    pub const PWM_L: u8 = 16; // left speed (pwm)
    pub const PWM_R: u8 = 13; // right speed (pwm)

    // led
    pub const LED_R: u8 = 22;
    pub const LED_G: u8 = 27;
    pub const LED_B: u8 = 24;

    // servo motor
    pub const SERVO: u8 = 23;

    // buzzer
    pub const BUZZER: u8 = 8;

    // infrared sensor pins
    pub const TRACK_L1: u8 = 3;
    pub const TRACK_L2: u8 = 5;
    pub const TRACK_R1: u8 = 4;
    pub const TRACK_R2: u8 = 18;

    // ultrasonic
    pub const ECHO: u8 = 0;
    pub const TRIG: u8 = 1;
}

pub enum LedColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
}
impl From<LedColor> for (bool, bool, bool) {
    fn from(value: LedColor) -> Self {
        match value {
            // 000
            LedColor::Black => (false, false, false),
            // 001
            LedColor::Red => (true, false, false),
            // 010
            LedColor::Green => (false, true, false),
            // 011
            LedColor::Yellow => (true, true, false),
            // 100
            LedColor::Blue => (false, false, true),
            // 101
            LedColor::Magenta => (true, false, true),
            // 110
            LedColor::Cyan => (false, true, true),
            // 111
            LedColor::White => (true, true, true),
        }
    }
}

struct Leds {
    r: OutputPin,
    g: OutputPin,
    b: OutputPin,
}

struct TrackSensor {
    l1: InputPin,
    l2: InputPin,
    r1: InputPin,
    r2: InputPin,
}

struct Motor {
    fwd_l: OutputPin,
    bwd_l: OutputPin,
    pwm_l: OutputPin,
    fwd_r: OutputPin,
    bwd_r: OutputPin,
    pwm_r: OutputPin,
}

struct UltraSensor {
    trig: OutputPin,
    echo: InputPin,
}

pub struct GPIORoland {
    ultra_sensor: Mutex<UltraSensor>,
    track_sensor: Mutex<TrackSensor>,
    buzzer: Mutex<OutputPin>,
    servo: Mutex<OutputPin>,
    motor: Mutex<Motor>,
    leds: Mutex<Leds>,
}

impl Drop for GPIORoland {
    fn drop(&mut self) {
        self.cleanup().expect("Failed to clean up!!!");
    }
}

impl GPIORoland {
    pub fn try_init() -> Result<GPIORoland> {
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

        let roland = GPIORoland {
            leds: Leds {
                r: gpio.get(LED_R)?.into_output(),
                g: gpio.get(LED_G)?.into_output(),
                b: gpio.get(LED_B)?.into_output(),
            }
            .into(),

            servo: gpio.get(SERVO)?.into_output().into(),

            buzzer: gpio.get(BUZZER)?.into_output().into(),

            motor: Motor {
                fwd_l: gpio.get(FWD_L)?.into_output(),
                bwd_l: gpio.get(BWD_L)?.into_output(),
                pwm_l: gpio.get(PWM_L)?.into_output(),
                fwd_r: gpio.get(FWD_R)?.into_output(),
                bwd_r: gpio.get(BWD_R)?.into_output(),
                pwm_r: gpio.get(PWM_R)?.into_output(),
            }
            .into(),

            track_sensor: TrackSensor {
                l1: gpio.get(TRACK_L1)?.into_input(),
                l2: gpio.get(TRACK_L2)?.into_input(),
                r1: gpio.get(TRACK_R1)?.into_input(),
                r2: gpio.get(TRACK_R2)?.into_input(),
            }
            .into(),

            ultra_sensor: UltraSensor {
                echo: gpio.get(ECHO)?.into_input(),
                trig: gpio.get(TRIG)?.into_output(),
            }
            .into(),
        };

        // ran here as well to reset servo to center
        roland.cleanup()?;

        Ok(roland)
    }

    #[cfg(feature = "camloc")]
    fn get_motion_hint(
        left: f64,
        left_sign: isize,
        right: f64,
        right_sign: isize,
    ) -> Option<MotionHint> {
        match (left_sign, right_sign) {
            (1, 1) | (1, 0) | (0, 1) => Some(MotionHint::MovingForwards),

            (0, 0) => Some(MotionHint::Stationary),

            (-1, -1) | (-1, 0) | (0, -1) => Some(MotionHint::MovingBackwards),

            // turning in place
            (1, -1) | (-1, 1) if (left * 100.) as usize == (-right * 100.) as usize => None,

            _ => unreachable!(),
        }
    }
}

impl Roland for GPIORoland {
    fn drive(&self, left: f64, right: f64) -> Result<DriveResult> {
        let left = clamp(left, -1., 1.);
        let right = clamp(right, -1., 1.);
        let mut m = self.motor.lock().unwrap();

        m.pwm_l.set_pwm_frequency(2000.0, left.abs())?;
        m.pwm_r.set_pwm_frequency(2000.0, right.abs())?;

        let left_sign = (left as isize).signum();
        match left_sign {
            1 => {
                m.fwd_l.set_high();
                m.bwd_l.set_low();
            }
            -1 => {
                m.fwd_l.set_low();
                m.bwd_l.set_high();
            }
            0 => {
                m.fwd_l.set_low();
                m.bwd_l.set_low();
            }
            _ => unreachable!(),
        }

        let right_sign = (right as isize).signum();
        match right_sign {
            1 => {
                m.fwd_r.set_high();
                m.bwd_r.set_low();
            }
            -1 => {
                m.fwd_r.set_low();
                m.bwd_r.set_high();
            }
            0 => {
                m.fwd_r.set_low();
                m.bwd_r.set_low();
            }
            _ => unreachable!(),
        }

        #[cfg(feature = "camloc")]
        let ret = Ok(Self::get_motion_hint(left, left_sign, right, right_sign));
        #[cfg(not(feature = "camloc"))]
        let ret = Ok(());

        ret
    }

    fn drive_by_angle(&self, angle: f64, speed: f64) -> Result<DriveResult> {
        let angle = clamp(angle, -90.0, 90.0);
        let speed = clamp(speed, -1., 1.);

        let a = (angle + 90.0) / 180.0;

        let left = (a * 100.0) * speed;
        let right = (100.0 - (a * 100.0)) * speed;

        self.drive(left, right)
    }

    fn led(&self, r: bool, g: bool, b: bool) -> Result<()> {
        let mut leds = self.leds.lock().unwrap();

        if r {
            leds.r.set_high();
        } else {
            leds.r.set_low();
        }
        if g {
            leds.g.set_high();
        } else {
            leds.g.set_low();
        }
        if b {
            leds.b.set_high();
        } else {
            leds.b.set_low();
        }

        Ok(())
    }

    fn servo(&self, degree: f64) -> Result<()> {
        let (period, pulse_width) = get_servo_pwm_durations(degree);
        self.servo.lock().unwrap().set_pwm(period, pulse_width)?;
        Ok(())
    }

    fn buzzer(&self, pw: f64) -> Result<()> {
        let mut pin = self.buzzer.lock().unwrap();

        if pw == 100.0 {
            pin.clear_pwm()?;
            pin.set_high();
        } else {
            pin.set_pwm_frequency(100.0, pw / 100.0)?;
        }

        Ok(())
    }

    fn track_sensor(&self) -> Result<[bool; 4]> {
        let s = self.track_sensor.lock().unwrap();
        Ok([
            s.l1.is_high(),
            s.l2.is_high(),
            s.r1.is_high(),
            s.r2.is_high(),
        ])
    }

    fn ultra_sensor(&self) -> Result<f64> {
        const BLAST_DURATION: Duration = Duration::from_micros(15);
        const CONVERSION_FACTOR: f64 = 340. / 2. * 100.;

        let mut s = self.ultra_sensor.lock().unwrap();

        s.trig.set_high();
        std::thread::sleep(BLAST_DURATION);
        s.trig.set_low();

        while s.echo.is_low() {}

        let t1 = Instant::now();
        while s.echo.is_high() {}
        let t2 = Instant::now();

        Ok((t2 - t1).as_secs_f64() * CONVERSION_FACTOR)
    }
}

#[cfg(feature = "camloc")]
pub type DriveResult = Option<MotionHint>;
#[cfg(not(feature = "camloc"))]
pub type DriveResult = ();

pub trait Roland: Sized {
    fn drive(&self, left: f64, right: f64) -> Result<DriveResult>;
    fn drive_by_angle(&self, angle: f64, speed: f64) -> Result<DriveResult>;
    fn led(&self, r: bool, g: bool, b: bool) -> Result<()>;
    fn servo(&self, degree: f64) -> Result<()>;
    fn buzzer(&self, pw: f64) -> Result<()>;
    fn track_sensor(&self) -> Result<[bool; 4]>;
    fn ultra_sensor(&self) -> Result<f64>;

    fn led_color(&self, color: LedColor) -> Result<()> {
        let (r, g, b) = color.into();
        self.led(r, g, b)
    }

    fn stop(&self) -> Result<()> {
        self.drive(0., 0.)?;
        Ok(())
    }

    fn cleanup(&self) -> Result<()> {
        self.drive(0., 0.)?;
        self.led(false, false, false)?;
        self.servo(0.)?;
        self.buzzer(100.0)?;

        Ok(())
    }
}
