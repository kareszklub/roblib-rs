use std::time::Duration;

use crate::{constants::*, util::clamp};
use rppal::gpio::Mode;
pub use rppal::gpio::{Error, Gpio};
pub type Result<T> = std::result::Result<T, Error>;

pub fn try_init(gpio: &Gpio) -> Result<()> {
    // will attempt to initialize all the pins just to see if they work
    gpio.get(LED_R)?;
    gpio.get(LED_G)?;
    gpio.get(LED_B)?;

    Ok(())
}

pub fn cleanup(gpio: &Gpio) -> Result<()> {
    gpio.get(LED_R)?.into_output().set_low();
    gpio.get(LED_G)?.into_output().set_low();
    gpio.get(LED_B)?.into_output().set_low();

    Ok(())
}

pub fn led(gpio: &Gpio, r: bool, g: bool, b: bool) -> Result<()> {
    let mut pin_r = gpio.get(LED_R)?.into_output();
    let mut pin_g = gpio.get(LED_G)?.into_output();
    let mut pin_b = gpio.get(LED_B)?.into_output();

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

    Ok(())
}

// TODO
pub fn buzzer(gpio: &Gpio, pw: f64) -> Result<()> {
    // let mut pin = gpio.get(BUZZER)?.into_output();
    let mut pin = gpio.get(BUZZER)?.into_io(Mode::Alt1);
    pin.set_reset_on_drop(false);
    dbg!(&pw);
    pin.set_pwm_frequency(50.0, pw)?;

    Ok(())
}

pub fn servo(gpio: &Gpio, degree: f64) -> Result<()> {
    let mut pin = gpio.get(SERVO)?.into_output();
    pin.set_reset_on_drop(false);

    let degree = 15.0 - (clamp(degree, -90.0, 90.0) / 9.0);
    pin.set_pwm_frequency(50.0, degree)?;

    // let degree = (degree as u64 * 11) + 50;
    // pin.set_pwm(Duration::from_millis(20), Duration::from_micros(degree))?;

    dbg!(&degree);

    Ok(())
}
