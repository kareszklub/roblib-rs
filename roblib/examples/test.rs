use anyhow::Result;
use roblib::gpio::roland::servo;
use std::{thread::sleep, time::Duration};

fn main() -> Result<()> {
    loop {
        for i in -90..90 {
            servo(i)?;
            sleep(Duration::from_millis(20));
        }
        for i in (-90..90).rev() {
            servo(i)?;
            sleep(Duration::from_millis(20));
        }
    }
}
