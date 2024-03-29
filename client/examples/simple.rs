use roblib::roland::{LedColor, Roland};
use roblib_client::{transports::tcp::Tcp, Result, Robot};
use std::{thread::sleep, time::Duration};

fn main() -> Result<()> {
    let ip = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "localhost:1110".into());

    let robot = Robot::new(Tcp::connect(ip)?);

    println!("Leds");
    robot.led_color(LedColor::Magenta)?;

    println!("Drive");
    robot.drive(0.4, 0.4)?;

    println!("Waiting...");
    sleep(Duration::from_secs(10));

    println!("Stopping");
    robot.stop()?;

    println!("Waiting...");
    sleep(Duration::from_secs(2));

    println!("Drive");
    robot.drive(0.4, 0.4)?;

    println!("Waiting...");
    sleep(Duration::from_secs(10));

    println!("Stopping");
    robot.stop()?;

    println!("Track sensor:");
    let data = robot.track_sensor()?;
    println!("    {data:?}");

    println!("Turning off leds");
    robot.led_color(LedColor::Black)?;

    #[cfg(feature = "camloc")]
    {
        use roblib::camloc::Camloc;
        println!("Position");
        if let Some(pos) = robot.get_position()? {
            println!("{pos}");
        } else {
            println!("Couldn't get position")
        }
    }

    Ok(())
}
