use roblib::gpio::roland::Robot;
use roblib_client::{http::RobotHTTP, logger::init_log, sleep, RemoteRobot, Result};
use std::time::Duration;

#[roblib_client::main]
async fn main() -> Result<()> {
    init_log(Some("roblib_client=debug"));

    let robot = RobotHTTP::connect("http://localhost:1111").await?;

    robot.led(true, false, false)?;

    robot.drive(40., 40.)?;

    sleep(Duration::from_secs(2)).await;

    robot.stop()?;

    println!("{:?}", robot.measure_latency()?);

    Ok(())
}
