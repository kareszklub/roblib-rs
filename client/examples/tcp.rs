use roblib::roland::Roland;
use roblib_client::{transports::tcp::Tcp, Result, Robot};
use std::{thread::sleep, time::Duration};

fn main() -> Result<()> {
    let ip = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "localhost:1110".into());

    let robot = Robot::new(Tcp::connect(ip)?);

    let up = robot.drive(0., 0.)?;
    dbg!(up);

    Ok(())
}
