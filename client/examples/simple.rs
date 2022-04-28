use roblib_client::{sleep, ws::Robot, Result};
use std::time::Duration;

#[roblib_client::main]
async fn main() -> Result<()> {
    let mut robot = Robot::connect("ws://localhost:1111/ws").await?;
    robot.led((true, false, true)).await?;

    robot.move_robot(10, 10).await?;

    sleep(Duration::from_secs(12)).await;

    robot.stop_robot().await?;

    let data = robot.get_sensor_data().await?;
    println!("{:?}", data);

    Ok(())
}
