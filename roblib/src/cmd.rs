#[cfg(unix)]
use crate::gpio::roland;
use anyhow::anyhow;
use std::{
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

pub enum Cmd {
    /// m
    MoveRobot(i8, i8),
    /// s
    StopRobot,
    /// l
    Led(bool, bool, bool),
    /// v
    ServoAbsolute(i8),
    /// b
    Buzzer(f64),
    /// t
    TrackSensor,
    /// z
    GetTime,
}

impl Cmd {
    pub fn exec(&self) -> anyhow::Result<Option<String>> {
        let res = match self {
            Cmd::MoveRobot(left, right) => {
                debug!("Moving robot: {left}:{right}");
                #[cfg(unix)]
                roland::drive(*left, *right)?;
                None
            }
            Cmd::StopRobot => {
                debug!("Stopping robot");
                #[cfg(unix)]
                roland::drive(0, 0)?;
                None
            }
            Cmd::Led(r, g, b) => {
                debug!("LED: {r}:{g}:{b}");
                #[cfg(unix)]
                roland::led(*r, *g, *b)?;
                None
            }
            Cmd::ServoAbsolute(deg) => {
                debug!("Servo absolute: {deg}");
                #[cfg(unix)]
                roland::servo(*deg)?;
                None
            }
            Cmd::Buzzer(pw) => {
                debug!("Buzzer: {pw}");
                #[cfg(unix)]
                roland::buzzer(*pw)?;
                None
            }
            Cmd::TrackSensor => {
                debug!("Track sensor");
                #[cfg(unix)]
                let res = roland::track_sensor()?;
                #[cfg(not(unix))]
                let res = [false, false, false, false];
                Some(format!("{},{},{},{}", res[0], res[1], res[2], res[3]))
            }
            Cmd::GetTime => Some(format!("{:.3}", get_time())),
        };
        Ok(res)
    }

    pub fn exec_str(s: &str) -> String {
        match Cmd::from_str(s).and_then(|c| c.exec()) {
            Ok(r) => r.unwrap_or_else(|| "OK".into()),
            Err(e) => e.to_string(),
        }
        // Cmd::from_str(s)?.exec()
    }
}

macro_rules! parse {
    ($args:ident $l:literal) => {{
        if $args.len() != $l {
            Err(anyhow!(
                "invalid number of arguments: expected {} got {}",
                $l,
                $args.len()
            ))?
        }

        $args
            .iter()
            .map(|s| s.parse())
            .collect::<Result<Vec<_>, _>>()?
    }};
}

impl FromStr for Cmd {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.split_whitespace().peekable();

        let c = iter.next().ok_or_else(|| anyhow!("missing command"))?;
        let args = iter.collect::<Vec<_>>();

        let res = match c {
            "m" => {
                let x = parse!(args 2);
                Cmd::MoveRobot(x[0], x[1])
            }
            "s" => Cmd::StopRobot,
            "l" => {
                let x = parse!(args 3);
                Cmd::Led(x[0], x[1], x[2])
            }
            "v" => {
                let x = parse!(args 1);
                Cmd::ServoAbsolute(x[0])
            }
            "t" => Cmd::TrackSensor,
            "b" => {
                let x = parse!(args 1);
                Cmd::Buzzer(x[0])
            }
            "z" => Cmd::GetTime,

            _ => Err(anyhow!("invalid command"))?,
        };
        Ok(res)
    }
}

pub type SensorData = [bool; 4];
// parse incoming data for the client
pub fn parse_sensor_data(s: &str) -> SensorData {
    s.split(',')
        .map(|s| {
            if s == "1" {
                true
            } else if s == "0" {
                false
            } else {
                panic!("invalid number")
            }
        })
        .collect::<Vec<_>>()
        .try_into()
        .unwrap_or_else(|v: Vec<_>| panic!("Expected a Vec of length {} but it was {}", 4, v.len()))
}

pub fn get_time() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards")
        .as_micros() as f64
        / 1000f64
}
