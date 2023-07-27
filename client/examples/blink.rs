use roblib::gpio::{Gpio, Mode};
use roblib_client::{logger::init_log, transports::udp::Udp, Result, Robot};
use std::{thread::sleep, time::Duration};

const P: u8 = 3;

fn main() -> Result<()> {
    init_log(Some("debug"));

    let ip = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "localhost:1110".into());

    let robot = Robot::new(Udp::connect(ip)?);

    log::info!("set pin mode");
    robot.pin_mode(P, Mode::Output)?;
    let mut state = false;
    loop {
        state = !state;
        robot.write_pin(P, state)?;
        sleep(Duration::from_secs(1));
    }
}
