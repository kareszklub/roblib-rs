use roblib::roland::Roland;
use roblib_client::{http::RobotHTTP, logger::init_log, RemoteRobotTransport, Result, Robot};
use std::{thread::sleep, time::Duration};

fn main() -> Result<()> {
    init_log(Some("roblib_client=debug"));

    let robot = Robot::new(RobotHTTP::create("http://localhost:1111")?);

    println!("Leds");
    robot.led(true, false, false)?;

    println!("Drive");
    robot.drive(40., 40.)?;

    println!("Waiting...");
    sleep(Duration::from_secs(2));

    println!("Stopping...");
    robot.stop()?;

    println!("Latency");
    println!("{:?}", robot.transport.measure_latency()?);

    Ok(())
}
