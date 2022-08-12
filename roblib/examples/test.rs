use anyhow::Result;

#[cfg(not(windows))]
fn main() -> Result<()> {
    use roblib::gpio::roland::servo;
    use std::{thread::sleep, time::Duration};

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

#[cfg(windows)]
fn main(){}

