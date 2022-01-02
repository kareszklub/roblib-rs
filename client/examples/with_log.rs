use roblib_client::{logger, Result, Robot};
use std::{thread::sleep, time::Duration};

#[actix_web::main]
async fn main() -> Result {
    logger::init_log(Some("roblib_client=debug"));

    let mut robot = Robot::connect("ws://localhost:8080/ws").await?;
    robot.led((true, false, true)).await?;

    robot.move_robot(10, 10).await?;

    sleep(Duration::from_secs(15));

    robot.stop_robot().await?;

    let data = robot.get_sensor_data().await?;
    println!("{:?}", data);

    Ok(())
}
