use roblib_client::{http::Robot, logger::init_log, sleep, Result};
use std::time::Duration;

#[roblib_client::main]
async fn main() -> Result<()> {
    init_log(Some("roblib_client=debug"));

    let robot = Robot::new("http://localhost:1111");
    robot.led((true, false, true)).await?;

    robot.move_robot(10, 10).await?;

    sleep(Duration::from_secs(12)).await;

    robot.stop_robot().await?;

    let data = robot.get_sensor_data().await?;
    println!("{:?}", data);

    Ok(())
}
