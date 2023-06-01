use roblib::gpio::roland::Roland;
use roblib_client::{sleep, ws::RobotWS, Result, Robot};
use std::time::Duration;

#[roblib_client::main]
async fn main() -> Result<()> {
    let robot = Robot::new(RobotWS::connect("ws://localhost:1111/ws").await?);

    robot.led(true, false, false)?;

    robot.drive(40., 40.)?;

    sleep(Duration::from_secs(2)).await;

    robot.stop()?;

    let data = robot.track_sensor()?;
    println!("{data:?}");

    let pos = robot.get_position()?;
    if let Some(pos) = pos {
        println!("{pos}");
    } else {
        println!("Couldn't get position")
    }

    Ok(())
}
