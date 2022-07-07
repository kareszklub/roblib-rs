// ! NOTE: both are very inaccurate.
use roland::gpio::Result;
use rppal::gpio::{Gpio, Trigger};

fn main() -> Result<()> {
    let gpio = Gpio::new()?;
    let mut pin = gpio.get(8)?.into_input_pulldown();

    // sync (blocking)
    {
        pin.set_interrupt(Trigger::Both)?;
        loop {
            let res = pin.poll_interrupt(false, None)?;
            dbg!(res);
        }
    }

    // async
    // {
    //     use std::{thread, time::Duration};
    //     pin.set_async_interrupt(Trigger::Both, |l| {
    //         dbg!(l);
    //     })?;
    //     thread::sleep(Duration::from_secs(100)); // do not exit instantly
    //     Ok(())
    // }
}
