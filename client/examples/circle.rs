use roblib::roland::{LedColor, Roland};
use roblib_client::{transports::tcp::Tcp, Result, Robot};
use std::{thread::sleep, time::Duration};

fn main() -> Result<()> {
    let ip = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "localhost:1110".into());

    let robot = Robot::new(Tcp::connect(ip)?);
    // let robot = Robot::new(Udp::connect(ip)?);

    println!("Leds");
    robot.led_color(LedColor::Magenta)?;

    println!("Drive");
    robot.drive(-0.2, 0.5)?;

    println!("Waiting...");
    sleep(Duration::from_secs(5));

    println!("Stopping");
    robot.stop()?;

    Ok(())
}
