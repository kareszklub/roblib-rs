use roblib::roland::RolandAsync;
use roblib_client::{async_robot::RobotAsync, transports::tcp::TcpAsync};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let ip = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "localhost:1110".into());

    let robot = RobotAsync::new(TcpAsync::connect(&ip).await?);

    robot.led(false, false, true).await?;

    robot.roland_servo(0.).await?;

    sleep(Duration::from_secs(1)).await;

    loop {
        robot.roland_servo(90.).await?;
        sleep(Duration::from_millis(1000)).await;
        robot.roland_servo(-90.).await?;
        sleep(Duration::from_millis(1000)).await;
    }
}
