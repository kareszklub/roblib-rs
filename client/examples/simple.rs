use roblib_client::{ws::Robot, Result};
use std::{thread::sleep, time::Duration};

#[actix_web::main]
async fn main() -> Result<()> {
    let mut robot = Robot::connect("ws://localhost:1111/ws").await?;
    robot.led((true, false, true)).await?;

    robot.move_robot(10, 10).await?;

    sleep(Duration::from_secs(2));

    robot.stop_robot().await?;

    let data = robot.get_sensor_data().await?;
    println!("{:?}", data);

    Ok(())
}
