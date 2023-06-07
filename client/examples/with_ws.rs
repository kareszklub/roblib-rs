use roblib::roland::Roland;
use roblib_client::{logger, ws::RobotWS, Result, Robot};
use std::{thread::sleep, time::Duration};

fn main() -> Result<()> {
    logger::init_log(Some("roblib_client=debug"));

    let robot = Robot::new(RobotWS::create("ws://localhost:1111/ws")?);

    println!("Leds");
    robot.led(true, false, false)?;

    println!("Drive");
    robot.drive(40., 40.)?;

    println!("Waiting...");
    sleep(Duration::from_secs(2));

    println!("Stop");
    robot.stop()?;

    println!("Track sensor:");
    let data = robot.track_sensor()?;
    println!("    {:?}", data);

    Ok(())
}
