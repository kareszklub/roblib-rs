use roblib::roland::Roland;
use roblib_client::{ws::RobotWS, Result, Robot};
use std::{thread::sleep, time::Duration};

fn main() -> Result<()> {
    let robot = Robot::new(RobotWS::create("ws://localhost:1111/ws")?);

    robot.led(true, false, false)?;

    robot.drive(40., 40.)?;

    sleep(Duration::from_secs(2));

    robot.stop()?;

    let data = robot.track_sensor()?;
    println!("track sensor: {data:?}");

    #[cfg(feature = "camloc")]
    {
        if let Some(pos) = robot.get_position()? {
            println!("{pos}");
        } else {
            println!("Couldn't get position")
        }
    }

    Ok(())
}
