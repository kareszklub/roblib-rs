use roblib_client::{Result, Robot};
use std::{
    thread::sleep,
    time::{Duration, Instant},
};

const NO_OF_RUNS: u64 = 100;
const WAIT_MS: u64 = 100;

#[actix_web::main]
async fn main() -> Result {
    // roblib_client::logger::init_log(Some("roblib_client=debug")); // uncomment if you want to spam the terminal
    let mut robot = Robot::connect("ws://localhost:1111/ws").await?;

    println!(
        "Starting test with {} runs, in intervals of {}ms",
        NO_OF_RUNS, WAIT_MS
    );
    let start = Instant::now();

    let mut v = Vec::new();
    for _ in 0..NO_OF_RUNS {
        let r = robot.measure_latency().await?;
        v.push(r);
        sleep(Duration::from_millis(WAIT_MS));
    }
    let sum = v.iter().sum::<f64>();
    let min = v
        .iter()
        .map(|x| *x)
        .reduce(f64::min)
        .expect("results contained NaN");
    let max = v
        .iter()
        .map(|x| *x)
        .reduce(f64::max)
        .expect("results contained NaN");
    let avg = sum / v.len() as f64;
    let dur = Instant::now().duration_since(start).as_millis() as f64 / 1000f64;
    println!(
        "Results:\n{:?}\nRuns: {}\nTime elapsed: {}s\nMin: {}ms\nMax: {}ms\nAverage: {:.3}ms",
        v, NO_OF_RUNS, dur, min, max, avg
    );

    Ok(())
}
