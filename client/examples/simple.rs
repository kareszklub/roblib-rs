use roblib::roland::{LedColor, Roland};
use roblib_client::{ws::RobotWS, Result, Robot};
use std::{thread::sleep, time::Duration};

fn main() -> Result<()> {
    let robot = Robot::new(RobotWS::create("ws://localhost:1111/ws")?);

    println!("Leds");
    robot.led_color(LedColor::Magenta)?;

    println!("Drive");
    robot.drive(40., 40.)?;

    println!("Waiting...");
    sleep(Duration::from_secs(2));

    println!("Stopping");
    robot.stop()?;

    println!("Drive");
    robot.drive(40., 40.)?;

    println!("Waiting...");
    sleep(Duration::from_secs(2));

    println!("Stopping");
    robot.stop()?;

    println!("Track sensor:");
    let data = robot.track_sensor()?;
    println!("    {data:?}");

    #[cfg(feature = "camloc")]
    {
        println!("Position");
        if let Some(pos) = robot.get_position()? {
            println!("{pos}");
        } else {
            println!("Couldn't get position")
        }
    }

    Ok(())
}
