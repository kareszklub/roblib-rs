use anyhow::{anyhow, Result};
use std::{fmt::Display, str::FromStr};

#[derive(Debug, PartialEq)]
pub enum Cmd {
    /// m
    #[cfg(feature = "roland")]
    MoveRobot(f64, f64),
    /// M
    #[cfg(feature = "roland")]
    MoveRobotByAngle(f64, f64),
    /// s
    #[cfg(feature = "roland")]
    StopRobot,
    /// l
    #[cfg(feature = "roland")]
    Led(bool, bool, bool),
    /// v
    #[cfg(feature = "roland")]
    ServoAbsolute(f64),
    /// b
    #[cfg(feature = "roland")]
    Buzzer(f64),
    /// t
    #[cfg(feature = "roland")]
    TrackSensor,
    /// u
    #[cfg(feature = "roland")]
    UltraSensor,

    /// r
    #[cfg(feature = "gpio")]
    ReadPin(u8),
    /// p
    #[cfg(feature = "gpio")]
    SetPin(u8, bool),
    /// w
    #[cfg(feature = "gpio")]
    SetPwm(u8, f64, f64),
    /// V
    #[cfg(feature = "gpio")]
    ServoBasic(u8, f64),

    /// P
    #[cfg(feature = "camloc")]
    GetPosition,

    /// n
    Nop,
    /// z
    GetUptime,
}

impl Display for Cmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "roland")]
            Cmd::MoveRobot(left, right) => write!(f, "m {left} {right}"),

            #[cfg(feature = "roland")]
            Cmd::MoveRobotByAngle(angle, speed) => write!(f, "M {angle} {speed}"),

            #[cfg(feature = "roland")]
            Cmd::StopRobot => write!(f, "s"),

            #[cfg(feature = "roland")]
            Cmd::Led(r, g, b) => write!(f, "l {} {} {}", *r as u8, *g as u8, *b as u8),

            #[cfg(feature = "roland")]
            Cmd::ServoAbsolute(deg) => write!(f, "v {deg}"),

            #[cfg(feature = "roland")]
            Cmd::Buzzer(pw) => write!(f, "b {pw}"),

            #[cfg(feature = "roland")]
            Cmd::TrackSensor => write!(f, "t"),

            #[cfg(feature = "roland")]
            Cmd::UltraSensor => write!(f, "u"),

            #[cfg(feature = "gpio")]
            Cmd::ReadPin(pin) => write!(f, "r {pin}"),

            #[cfg(feature = "gpio")]
            Cmd::SetPin(pin, value) => write!(f, "p {} {}", pin, *value as u8),

            #[cfg(feature = "gpio")]
            Cmd::SetPwm(pin, hz, cycle) => write!(f, "w {pin} {hz} {cycle}"),

            #[cfg(feature = "gpio")]
            Cmd::ServoBasic(pin, deg) => write!(f, "V {pin} {deg}"),

            #[cfg(feature = "camloc")]
            Cmd::GetPosition => write!(f, "P"),

            Cmd::Nop => write!(f, "n"),

            Cmd::GetUptime => write!(f, "z"),
        }
    }
}

fn assert_args_len<const N: usize, T>(slice: &[T]) -> Result<()> {
    let len = slice.len();
    if len == N {
        Ok(())
    } else {
        Err(anyhow!(
            "invalid number of arguments: expected {N} got {len}"
        ))?
    }
}
fn parse_args<const N: usize, T: FromStr + std::fmt::Debug>(args: &[&str]) -> Result<[T; N]> {
    assert_args_len::<N, &str>(args)?;
    Ok(args
        .iter()
        .map(|s| s.parse())
        .collect::<std::result::Result<Vec<T>, _>>()
        .map_err(|_| anyhow!("Couldn't parse one of the arguments"))?
        .try_into()
        .unwrap())
}

impl FromStr for Cmd {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.split_whitespace().peekable();
        let c = iter.next().ok_or_else(|| anyhow!("missing command"))?;

        #[allow(unused)]
        let args: Vec<&str> = iter.collect();

        let res = match c {
            #[cfg(feature = "roland")]
            "m" => {
                let x: [f64; 2] = parse_args(&args)?;
                Cmd::MoveRobot(x[0], x[1])
            }

            #[cfg(feature = "roland")]
            "M" => {
                let x: [f64; 2] = parse_args(&args)?;
                Cmd::MoveRobotByAngle(x[0], x[1])
            }

            #[cfg(feature = "roland")]
            "s" => Cmd::StopRobot,

            #[cfg(feature = "roland")]
            "l" => {
                let x: [u8; 3] = parse_args(&args)?;
                let x = x.map(|b| b != 0);
                Cmd::Led(x[0], x[1], x[2])
            }

            #[cfg(feature = "roland")]
            "v" => Cmd::ServoAbsolute(parse_args::<1, f64>(&args)?[0]),

            #[cfg(feature = "roland")]
            "t" => Cmd::TrackSensor,

            #[cfg(feature = "roland")]
            "b" => Cmd::Buzzer(parse_args::<1, f64>(&args)?[0]),

            #[cfg(feature = "roland")]
            "u" => Cmd::UltraSensor,

            #[cfg(feature = "gpio")]
            "r" => Cmd::ReadPin(parse_args::<1, u8>(&args)?[0]),

            #[cfg(feature = "gpio")]
            "p" => {
                let x: [u8; 2] = parse_args(&args)?;
                Cmd::SetPin(x[0], x[1] != 0)
            }

            #[cfg(feature = "gpio")]
            "w" => {
                assert_args_len::<3, &str>(&args)?;
                let x1: [u8; 1] = parse_args(&args[..1])?;
                let x2: [f64; 2] = parse_args(&args[1..])?;
                Cmd::SetPwm(x1[0], x2[0], x2[1])
            }

            #[cfg(feature = "gpio")]
            "V" => {
                assert_args_len::<2, &str>(&args)?;
                let x1: [u8; 1] = parse_args(&args[..1])?;
                let x2: [f64; 1] = parse_args(&args[1..])?;
                Cmd::ServoBasic(x1[0], x2[0])
            }

            #[cfg(feature = "camloc")]
            "P" => Cmd::GetPosition,

            "n" => Cmd::Nop,

            "z" => Cmd::GetUptime,

            _ => Err(anyhow!("invalid command"))?,
        };

        Ok(res)
    }
}

#[cfg(feature = "roland")]
pub fn parse_track_sensor_data(s: &str) -> Result<[bool; 4]> {
    let args: Vec<&str> = s.split(',').collect();
    let args: [u8; 4] = parse_args(&args)?;
    Ok(args.map(|byte| byte != 0))
}

#[cfg(feature = "camloc")]
pub fn parse_position_data(s: &str) -> Result<Option<camloc_server::Position>> {
    if s.is_empty() {
        return Ok(None);
    }
    let args: Vec<&str> = s.split(',').collect();
    let args: [f64; 3] = parse_args(&args)?;
    Ok(Some(camloc_server::Position::new(
        args[0], args[1], args[2],
    )))
}

#[cfg(feature = "roland")]
pub fn parse_ultra_sensor_data(s: &str) -> Result<f64> {
    s.parse().map_err(|_| anyhow!("expected a float"))
}

#[cfg(feature = "gpio")]
pub fn parse_pin_data(s: &str) -> Result<bool> {
    match s {
        "0" => Ok(false),
        "1" => Ok(true),
        _ => Err(anyhow!("expected 0 or 1")),
    }
}
