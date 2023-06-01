use roblib::gpio::roland::Roland;
use roblib_client::{
    http::RobotHTTP, logger::init_log, sleep, RemoteRobotTransport, Result, Robot,
};
use std::time::Duration;

#[roblib_client::main]
async fn main() -> Result<()> {
    init_log(Some("roblib_client=debug"));

    let robot = Robot::new(RobotHTTP::connect("http://localhost:1111").await?);

    robot.led(true, false, false)?;

    robot.drive(40., 40.)?;

    sleep(Duration::from_secs(2)).await;

    robot.stop()?;

    println!("{:?}", robot.transport.measure_latency()?);

    Ok(())
}
