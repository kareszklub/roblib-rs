use roblib_client::{http::Robot, logger::init_log, Result};

#[actix_web::main]
async fn main() -> Result<()> {
    init_log(Some("roblib_client=debug"));

    let robot = Robot::new("http://localhost:1111");
    robot.stop_robot().await?;

    Ok(())
}
