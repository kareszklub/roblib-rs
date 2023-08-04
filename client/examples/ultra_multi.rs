use roblib::event;
use roblib_client::{transports::tcp::Tcp, Result, Robot};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

fn main() -> Result<()> {
    roblib_client::logger::init_log(Some("debug"));
    let ip = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "localhost:1110".into());

    let robot = Arc::new(Robot::new(Tcp::connect(ip)?));

    let (tx, rx) = std::sync::mpsc::sync_channel(1);

    let tx1 = Box::leak(Box::new(tx.clone()));
    robot.subscribe(event::UltraSensor(Duration::from_millis(5000)), |_| {
        tx1.send(5)?;
        Ok(())
    })?;

    let tx2 = Box::leak(Box::new(tx));
    robot.subscribe(event::UltraSensor(Duration::from_millis(3000)), |_| {
        tx2.send(3)?;
        Ok(())
    })?;

    robot.subscribe(event::TrackSensor, |v| {
        println!("t | {v:?}");
        Ok(())
    })?;

    let mut prev = Instant::now();
    loop {
        let v = rx.recv()?;
        let now = Instant::now();
        println!("{v} | {:?}", now - prev);
        prev = now;
    }
}
