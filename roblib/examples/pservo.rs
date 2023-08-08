//! Testing controlling the servo without a pwm thread
use rppal::gpio::Gpio;
use std::{
    env::args,
    time::{Duration, Instant},
};

const SERVO: u8 = 23;

fn main() -> rppal::gpio::Result<()> {
    // dbg!(deg(-90), deg(0), deg(45), deg(90));

    let mut argv = args().skip(1);
    let m = argv.next().unwrap_or("1500".to_string()).parse().unwrap();
    //let d = argv.next().unwrap_or("0".to_string()).parse().unwrap();
    let n = argv.next().unwrap_or("1".to_string()).parse().unwrap();
    //   let range = argv.next().unwrap_or("0".to_string()).parse().unwrap();

    //    let dur = deg(d, range);
    let dur = Duration::from_micros(m);

    let gpio = Gpio::new()?;
    let mut pin = gpio.get(SERVO)?.into_output();

    for _ in 0..n {
        pin.set_high();
        let stop = Instant::now() + dur;
        while Instant::now() < stop {}
        pin.set_low();
        std::thread::sleep(Duration::from_millis(15));
    }

    Ok(())
}

#[allow(dead_code)]
fn deg(deg: f64, range: f64) -> Duration {
    let millis = ((deg + 90.) / 180.) + 1. + range;
    Duration::from_secs_f64(millis / 1000.)
}
