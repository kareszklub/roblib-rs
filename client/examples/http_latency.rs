use roblib_client::{http::Robot, sleep, Result};
use std::{
    env::args,
    time::{Duration, Instant},
};

const NO_OF_RUNS: usize = 25;
const WAIT_MS: u64 = 100;

#[actix_web::main]
async fn main() -> Result<()> {
    // roblib_client::logger::init_log(Some("roblib_client=debug")); // uncomment if you want to spam the terminal

    let ip = std::env::args().nth(1).unwrap_or("localhost:1111".into());
    let robot = Robot::new(&format!("http://{ip}"));

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
        sleep(Duration::from_millis(wait_ms)).await;
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
        "Results:\n{v:?}\nRuns: {runs}\nTime elapsed: {dur}s\nMin: {min:.3}ms\nMax: {max:.3}ms\nAverage: {avg:.3}ms",
        v=v.iter().map(|n|format!("{n:.3}").parse().unwrap()).collect::<Vec<f64>>()
    );

    Ok(())
}
