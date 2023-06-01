use roblib::gpio::roland::Robot;
use roblib_client::{logger, sleep, ws::RobotWS, Result};
use std::time::Duration;

#[roblib_client::main]
async fn main() -> Result<()> {
    logger::init_log(Some("roblib_client=debug"));

    let robot = RobotWS::connect("ws://localhost:1111/ws").await?;

    robot.led(true, false, false)?;

    robot.drive(40, 40)?;

    sleep(Duration::from_secs(2)).await;

    robot.stop()?;

    let data = robot.track_sensor()?;
    println!("{:?}", data);

    Ok(())
}
