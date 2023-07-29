use std::time::Duration;

use roblib::event;
use roblib_client::{transports::udp::Udp, Result, Robot};

fn main() -> Result<()> {
    let ip = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "localhost:1110".into());

    let robot = Robot::new(Udp::connect(ip)?);
    // let mut n = 0;

    robot.subscribe(event::UltraSensor(Duration::from_millis(100)), move |v| {
        // n += 1;
        if v < 2. {
            println!("{v:?}",);
        }
        Ok(())
    })?;

    loop {
        std::thread::park();
    }

    // loop {
    //     let track: Vec<_> = robot.track_sensor()?.iter().map(|b| *b as u8).collect();
    //     let ultra = robot.ultra_sensor()?;
    //     println!("Track sensor: {track:?} - Ultra sensor: {ultra:.3}");
    //     sleep(Duration::from_millis(10));
    // }
}
