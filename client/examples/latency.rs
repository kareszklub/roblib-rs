use roblib_client::{Result, Robot};
use std::{
    env::args,
    thread::sleep,
    time::{Duration, Instant},
};

const NO_OF_RUNS: usize = 25;
const WAIT_MS: u64 = 100;

#[actix_web::main]
async fn main() -> Result {
    // roblib_client::logger::init_log(Some("roblib_client=debug")); // uncomment if you want to spam the terminal
    let mut robot = Robot::connect("ws://localhost:1111/ws").await?;

    // boring arg parsing
    let mut args = args().skip(1);
    let runs = args
        .next()
        .unwrap_or(NO_OF_RUNS.to_string())
        .parse()
        .unwrap_or(NO_OF_RUNS);
    let wait_ms = args
        .next()
        .unwrap_or(WAIT_MS.to_string())
        .parse()
        .unwrap_or(WAIT_MS);

    println!(
        "Starting test with {} runs, in intervals of {}ms",
        runs, wait_ms
    );

    let start = Instant::now();
    let mut v = Vec::with_capacity(runs);
    for _ in 0..runs {
        let r = robot.measure_latency().await?;
        v.push(r);
        sleep(Duration::from_millis(wait_ms));
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
        v, runs, dur, min, max, avg
    );

    Ok(())
}
