use roblib::event;
use roblib_client::{transports::tcp::Tcp, Result, Robot};
use std::{sync::Arc, time::Duration};

fn main() -> Result<()> {
    roblib_client::logger::init_log(Some("debug"));
    let ip = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "localhost:1110".into());

    let robot = Arc::new(Robot::new(Tcp::connect(ip)?));
    // let mut n = 0;

    let ev = event::UltraSensor(Duration::from_millis(100));
    robot.subscribe(ev, move |v| {
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
    //     std::thread::sleep(Duration::from_millis(10));
    // }
}
