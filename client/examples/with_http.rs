use roblib::roland::Roland;
use roblib_client::{http::RobotHTTP, logger::init_log, RemoteRobotTransport, Result, Robot};
use std::{thread::sleep, time::Duration};

fn main() -> Result<()> {
    init_log(Some("roblib_client=debug"));

    let robot = Robot::new(RobotHTTP::create("http://localhost:1111")?);

    robot.led(true, false, false)?;

    robot.drive(40., 40.)?;

    sleep(Duration::from_secs(2));

    robot.stop()?;

    println!("{:?}", robot.transport.measure_latency()?);

    Ok(())
}
