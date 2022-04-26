pub mod constants;
use crate::constants::*;

pub use rppal::gpio::{Error, Gpio};
pub type Result<T> = std::result::Result<T, Error>;

pub fn try_init(gpio: &Gpio) -> Result<()> {
    // will attempt to initialize all the pins just to see if they work
    gpio.get(LED_R)?;
    gpio.get(LED_G)?;
    gpio.get(LED_B)?;

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
