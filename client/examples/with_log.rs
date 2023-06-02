use roblib::roland::Roland;
use roblib_client::{logger, ws::RobotWS, Result, Robot};
use std::{thread::sleep, time::Duration};

fn main() -> Result<()> {
    logger::init_log(Some("roblib_client=debug"));

    let robot = Robot::new(RobotWS::create("ws://localhost:1111/ws")?);

    robot.led(true, false, false)?;

    robot.drive(40., 40.)?;

    sleep(Duration::from_secs(2));

    robot.stop()?;

    let data = robot.track_sensor()?;
    println!("{:?}", data);

    Ok(())
}
