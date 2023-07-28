use super::Roland;
use crate::get_servo_pwm_durations;
use anyhow::Result;
use rppal::gpio::{Gpio, InputPin, OutputPin};
use std::{
    sync::Mutex,
    time::{Duration, Instant},
};

pub mod constants {
    pub mod motors {
        /// left forward
        pub const FWD_L: u8 = 20;
        /// left backward
        pub const BWD_L: u8 = 21;
        /// right forward
        pub const FWD_R: u8 = 19;
        /// right backward
        pub const BWD_R: u8 = 26;
        /// left speed (pwm)
        pub const PWM_L: u8 = 16;
        /// right speed (pwm)
        pub const PWM_R: u8 = 13;
    }

    pub mod led {
        pub const LED_R: u8 = 22;
        pub const LED_G: u8 = 27;
        pub const LED_B: u8 = 24;
    }

    pub mod servo {
        pub const SERVO: u8 = 23;
    }

    pub mod buzzer {
        pub const BUZZER: u8 = 8;
    }

    pub mod track_sensor {
        pub const TRACK_L1: u8 = 3;
        pub const TRACK_L2: u8 = 5;
        pub const TRACK_R1: u8 = 4;
        pub const TRACK_R2: u8 = 18;
    }

    pub mod ultra_sensor {
        use std::time::Duration;

        pub const ECHO: u8 = 0;
        pub const TRIG: u8 = 1;

        pub const BLAST_DURATION: Duration = Duration::from_micros(15);
        pub(in super::super) const CONVERSION_FACTOR: f64 = 340. / 2.;
    }
}

struct Leds {
    r: OutputPin,
    g: OutputPin,
    b: OutputPin,
}
impl Leds {
    fn new(gpio: &Gpio) -> Result<Self> {
        use constants::led::*;
        Ok(Self {
            r: gpio.get(LED_R)?.into_output_low(),
            g: gpio.get(LED_G)?.into_output_low(),
            b: gpio.get(LED_B)?.into_output_low(),
        })
    }
}

struct TrackSensor {
    l1: InputPin,
    l2: InputPin,
    r1: InputPin,
    r2: InputPin,
}
impl TrackSensor {
    fn new(gpio: &Gpio) -> Result<Self> {
        use constants::track_sensor::*;
        Ok(Self {
            l1: gpio.get(TRACK_L1)?.into_input(),
            l2: gpio.get(TRACK_L2)?.into_input(),
            r1: gpio.get(TRACK_R1)?.into_input(),
            r2: gpio.get(TRACK_R2)?.into_input(),
        })
    }
}

struct Buzzer(OutputPin);
impl Buzzer {
    pub fn new(gpio: &Gpio) -> Result<Self> {
        Ok(Self(
            gpio.get(constants::buzzer::BUZZER)?.into_output_high(),
        ))
    }
}

struct Servo(OutputPin);
impl Servo {
    pub fn new(gpio: &Gpio) -> Result<Self> {
        Ok(Self(gpio.get(constants::servo::SERVO)?.into_output_high()))
    }
}
struct Motors {
    fwd_l: OutputPin,
    bwd_l: OutputPin,
    pwm_l: OutputPin,

    fwd_r: OutputPin,
    bwd_r: OutputPin,
    pwm_r: OutputPin,
}
impl Motors {
    fn new(gpio: &Gpio) -> Result<Self> {
        use constants::motors::*;
        Ok(Self {
            fwd_l: gpio.get(FWD_L)?.into_output_low(),
            bwd_l: gpio.get(BWD_L)?.into_output_low(),
            pwm_l: gpio.get(PWM_L)?.into_output_high(),

            fwd_r: gpio.get(FWD_R)?.into_output_low(),
            bwd_r: gpio.get(BWD_R)?.into_output_low(),
            pwm_r: gpio.get(PWM_R)?.into_output_high(),
        })
    }
}

struct UltraSensor {
    trig: OutputPin,
    echo: InputPin,
}

impl UltraSensor {
    fn new(gpio: &Gpio) -> Result<Self> {
        use constants::ultra_sensor::{ECHO, TRIG};
        Ok(Self {
            echo: gpio.get(ECHO)?.into_input(),
            trig: gpio.get(TRIG)?.into_output_low(),
        })
    }
}

pub struct RolandBackend {
    ultra_sensor: Mutex<UltraSensor>,
    track_sensor: Mutex<TrackSensor>,
    buzzer: Mutex<Buzzer>,
    servo: Mutex<Servo>,
    motor: Mutex<Motors>,
    leds: Mutex<Leds>,

    gpio: Gpio,
}

impl Drop for RolandBackend {
    fn drop(&mut self) {
        self.cleanup().expect("Failed to clean up!!!");
    }
}

impl RolandBackend {
    pub fn try_init() -> Result<Self> {
        let gpio = Gpio::new()?;

        let roland = Self {
            motor: Motors::new(&gpio)?.into(),
            servo: Servo::new(&gpio)?.into(),

            buzzer: Buzzer::new(&gpio)?.into(),
            leds: Leds::new(&gpio)?.into(),

            ultra_sensor: UltraSensor::new(&gpio)?.into(),
            track_sensor: TrackSensor::new(&gpio)?.into(),

            gpio,
        };

        // ran here as well to reset servo to center
        roland.cleanup()?;

        Ok(roland)
    }

    pub fn setup_tracksensor_interrupts(&self) -> Result<()> {
        let mut s = self.track_sensor.lock().unwrap();

        s.l1.set_interrupt(rppal::gpio::Trigger::Both)?;
        s.l2.set_interrupt(rppal::gpio::Trigger::Both)?;
        s.r1.set_interrupt(rppal::gpio::Trigger::Both)?;
        s.r2.set_interrupt(rppal::gpio::Trigger::Both)?;

        Ok(())
    }

    pub fn poll_tracksensor(&self, timeout: Option<Duration>) -> Result<Option<(usize, bool)>> {
        let s = self.track_sensor.lock().unwrap();

        let ps = [&s.l1, &s.l2, &s.r1, &s.r2];
        let res = self.gpio.poll_interrupts(&ps, false, timeout)?;

        let Some((p, v)) = res else { return Ok(None) };

        Ok(Some((
            ps.iter().position(|a| *a == p).unwrap(),
            v as u8 != 0,
        )))
    }

    pub fn clear_tracksensor_interrupts(&self) -> Result<()> {
        let mut s = self.track_sensor.lock().unwrap();
        s.l1.clear_interrupt()?;
        s.l2.clear_interrupt()?;
        s.r1.clear_interrupt()?;
        s.r2.clear_interrupt()?;
        Ok(())
    }
}

impl Roland for RolandBackend {
    fn drive(&self, left: f64, right: f64) -> Result<()> {
        let left = left.clamp(-1., 1.);
        let right = right.clamp(-1., 1.);
        let mut m = self.motor.lock().unwrap();

        m.pwm_l.set_pwm_frequency(2000.0, left.abs())?;
        m.pwm_r.set_pwm_frequency(2000.0, right.abs())?;

        match left.signum() as isize {
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

        match right.signum() as isize {
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

        Ok(())
    }

    fn led(&self, r: bool, g: bool, b: bool) -> Result<()> {
        let mut leds = self.leds.lock().unwrap();

        leds.r.write(r.into());
        leds.g.write(g.into());
        leds.b.write(b.into());

        Ok(())
    }

    fn roland_servo(&self, degree: f64) -> Result<()> {
        let (period, pulse_width) = get_servo_pwm_durations(degree);
        self.servo.lock().unwrap().0.set_pwm(period, pulse_width)?;
        Ok(())
    }

    fn buzzer(&self, pw: f64) -> Result<()> {
        let mut pin = self.buzzer.lock().unwrap();
        let pin = &mut pin.0;

        let pw = pw.clamp(0., 1.);

        if pw >= 1. {
            pin.clear_pwm()?;
            pin.set_high();
        } else {
            pin.set_pwm_frequency(100.0, pw)?;
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
        use self::constants::ultra_sensor::{BLAST_DURATION, CONVERSION_FACTOR};
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
