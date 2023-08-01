use roblib::roland::{LedColor, RolandAsync};
use roblib_client::{
    async_robot::RobotAsync,
    transports::{http::Http, ws::Ws},
};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let ip = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "localhost:1111".into());

    // let robot = RobotAsync::new(Http::connect(&ip)?);
    let robot = RobotAsync::new(Ws::connect(&ip).await?);

    println!("Leds");
    robot.led_color(LedColor::Magenta).await?;

    println!("Drive");
    robot.drive(0.4, 0.4).await?;

    println!("Waiting...");
    sleep(Duration::from_secs(2)).await;

    println!("Stopping");
    robot.stop().await?;

    println!("Waiting...");
    sleep(Duration::from_secs(2)).await;

    println!("Drive");
    robot.drive(0.4, 0.4).await?;

    println!("Waiting...");
    sleep(Duration::from_secs(2)).await;

    println!("Stopping");
    robot.stop().await?;

    println!("Track sensor:");
    let data = robot.track_sensor().await?;
    println!("    {data:?}");

    println!("Turning off leds");
    robot.led_color(LedColor::Black).await?;

    #[cfg(feature = "camloc")]
    {
        use roblib::camloc::CamlocAsync;
        println!("Position");
        if let Some(pos) = robot.get_position().await? {
            println!("{pos}");
        } else {
            println!("Couldn't get position")
        }
    }

    Ok(())
}