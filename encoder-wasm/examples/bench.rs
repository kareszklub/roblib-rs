use roblib::cmd::Cmd;
use roblib::cmd::{binary_opts as opts, BincodeOptions};
use std::str::FromStr;

const RUNS: usize = 1000000;
static CMD: Cmd = Cmd::MoveRobotByAngle(std::f64::consts::PI - 100.0, 1, Some((true, true, true)));

fn main() {
    println!("{}", CMD.to_string());
    measure("encode_str", encode_str);
    measure("encode_bin", encode_bin);
    measure("decode_str", decode_str);
    measure("decode_bin", decode_bin);
}

fn measure(name: &str, f: impl FnOnce()) {
    let start = std::time::Instant::now();
    f();
    let elapsed = start.elapsed();
    println!(
        "{name}: {}.{:09} seconds",
        elapsed.as_secs(),
        elapsed.subsec_nanos()
    );
}

fn encode_str() {
    for _ in 0..RUNS {
        let _s = CMD.to_string();
    }
}

fn encode_bin() {
    for _ in 0..RUNS {
        let _bytes = opts().serialize(&CMD).unwrap();
    }
}

fn decode_str() {
    let s = CMD.to_string();
    for _ in 0..RUNS {
        let _cmd: Cmd = Cmd::from_str(&s).unwrap();
    }
}

fn decode_bin() {
    let bytes = opts().serialize(&CMD).unwrap();
    for _ in 0..RUNS {
        let _cmd: Cmd = opts().deserialize(&bytes).unwrap();
    }
}
