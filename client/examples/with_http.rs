use roblib_client::{cmd::Cmd, http::Robot, logger::init_log, sleep, Result};
use std::time::Duration;

#[roblib_client::main]
async fn main() -> Result<()> {
    init_log(Some("roblib_client=debug"));

    let robot = Robot::new("http://localhost:1111");

    robot.cmd(Cmd::Led(true, false, false)).await?;

    robot.cmd(Cmd::MoveRobot(40, 40)).await?;

    sleep(Duration::from_secs(2)).await;

    robot.cmd(Cmd::StopRobot).await?;

    let data = robot.get_sensor_data().await?;
    println!("{:?}", data);

    Ok(())
}
