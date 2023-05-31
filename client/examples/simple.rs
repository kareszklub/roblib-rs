use roblib_client::{cmd::Cmd, sleep, ws::Robot, Result};
use std::time::Duration;

#[roblib_client::main]
async fn main() -> Result<()> {
    let mut robot = Robot::connect("ws://localhost:1111/ws").await?;

    robot.cmd(Cmd::Led(true, false, false)).await?;

    robot.cmd(Cmd::MoveRobot(40, 40)).await?;

    sleep(Duration::from_secs(2)).await;

    robot.cmd(Cmd::StopRobot).await?;

    let data = robot.get_track_sensor_data().await?;
    println!("{data:?}");

    let pos = robot.get_position().await?;
    if let Some(pos) = pos {
        println!("{pos}");
    } else {
        println!("Couldn't get position")
    }

    Ok(())
}
