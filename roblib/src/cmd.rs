#[cfg(all(unix, feature = "gpio"))]
use crate::gpio::{self, roland::Roland};
use anyhow::{anyhow, Result};
use camloc_server::Position;
use std::{
    fmt::Display,
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, PartialEq)]
pub enum Cmd {
    /// m
    #[cfg(feature = "roland")]
    MoveRobot(i8, i8),
    /// M
    #[cfg(feature = "roland")]
    MoveRobotByAngle(f64, i8),
    /// s
    #[cfg(feature = "roland")]
    StopRobot,
    /// l
    #[cfg(feature = "roland")]
    Led(bool, bool, bool),
    /// v
    #[cfg(feature = "roland")]
    ServoAbsolute(i8),
    /// b
    #[cfg(feature = "roland")]
    Buzzer(f64),
    /// t
    #[cfg(feature = "roland")]
    TrackSensor,

    /// P
    #[cfg(feature = "roland")]
    GetPosition,

    /// p
    SetPin(u8, bool),
    /// w
    SetPwm(u8, f64, f64),
    /// V
    ServoBasic(u8, i8),
    /// z
    GetTime,
}

impl Cmd {
    #[allow(unused_variables)]
    pub fn exec(&self, _run: bool, roland: Option<&Roland>) -> anyhow::Result<Option<String>> {
        let res = match self {
            #[cfg(feature = "roland")]
            Cmd::MoveRobot(left, right) => {
                debug!("Moving robot: {left}:{right}");
                #[cfg(all(unix, feature = "gpio"))]
                if _run {
                    roland.unwrap().drive(*left, *right)?
                };
                None
            }
            #[cfg(feature = "roland")]
            Cmd::MoveRobotByAngle(angle, speed) => {
                debug!("Moving robot by angle: {}:{}", angle, speed);

                #[cfg(all(unix, feature = "gpio"))]
                if _run {
                    roland.unwrap().drive_by_angle(*angle, *speed)?
                };
                None
            }
            #[cfg(feature = "roland")]
            Cmd::StopRobot => {
                debug!("Stopping robot");
                #[cfg(all(unix, feature = "gpio"))]
                if _run {
                    roland.unwrap().drive(0, 0)?
                };
                None
            }
            #[cfg(feature = "roland")]
            Cmd::Led(r, g, b) => {
                debug!("LED: {r}:{g}:{b}");
                #[cfg(all(unix, feature = "gpio"))]
                if _run {
                    roland.unwrap().led(*r, *g, *b)?
                };
                None
            }
            #[cfg(feature = "roland")]
            Cmd::ServoAbsolute(deg) => {
                debug!("Servo absolute: {deg}");
                #[cfg(all(unix, feature = "gpio"))]
                if _run {
                    roland.unwrap().servo(*deg)?
                };
                None
            }
            #[cfg(feature = "roland")]
            Cmd::Buzzer(pw) => {
                debug!("Buzzer: {pw}");
                #[cfg(all(unix, feature = "gpio"))]
                if _run {
                    roland.unwrap().buzzer(*pw)?
                };
                None
            }
            #[cfg(feature = "roland")]
            Cmd::TrackSensor => {
                debug!("Track sensor");
                #[cfg(all(unix, feature = "gpio"))]
                let res = if _run {
                    roland.unwrap().track_sensor()?
                } else {
                    [false, false, false, false]
                };
                #[cfg(not(all(unix, feature = "gpio")))]
                let res = [false, false, false, false];
                Some(format!("{},{},{},{}", res[0], res[1], res[2], res[3]))
            }

            #[cfg(feature = "roland")]
            Cmd::GetPosition => {
                debug!("Get position");
                #[cfg(all(unix, feature = "gpio"))]
                let res = if _run {
                    roland.unwrap().get_position()
                } else {
                    None
                };

                #[cfg(not(all(unix, feature = "gpio")))]
                let res = None;

                Some(if let Some(pos) = res {
                    format!(
                        "{},{},{}",
                        pos.position.x, pos.position.y, pos.position.rotation
                    )
                } else {
                    String::new()
                })
            }

            Cmd::SetPin(pin, value) => {
                debug!("Set pin: {pin}:{value}");
                #[cfg(all(unix, feature = "gpio"))]
                gpio::set(*pin, *value)?;
                None
            }
            Cmd::SetPwm(pin, hz, cycle) => {
                debug!("Set pwm: {pin}:{hz}:{cycle}");
                #[cfg(all(unix, feature = "gpio"))]
                gpio::pwm(*pin, *hz, *cycle)?;
                None
            }
            Cmd::ServoBasic(pin, deg) => {
                debug!("Servo basic: {deg}");
                #[cfg(all(unix, feature = "gpio"))]
                gpio::servo(*pin, *deg)?;
                None
            }
            Cmd::GetTime => Some(format!("{:.3}", get_time()?)),
        };
        Ok(res)
    }

    pub fn exec_str(s: &str, run: bool, roland: Option<&Roland>) -> String {
        match Cmd::from_str(s).and_then(|c| c.exec(run, roland)) {
            Ok(r) => r.unwrap_or_else(|| "OK".into()),
            Err(e) => e.to_string(),
        }
        // Cmd::from_str(s)?.exec()
    }
}

impl Display for Cmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(feature = "roland")]
            Cmd::MoveRobot(left, right) => write!(f, "m {} {}", left, right),
            #[cfg(feature = "roland")]
            Cmd::MoveRobotByAngle(angle, speed) => write!(f, "M {} {} 0", angle, speed),
            #[cfg(feature = "roland")]
            Cmd::StopRobot => write!(f, "s"),
            #[cfg(feature = "roland")]
            Cmd::Led(r, g, b) => write!(f, "l {} {} {}", *r as u8, *g as u8, *b as u8),
            #[cfg(feature = "roland")]
            Cmd::ServoAbsolute(deg) => write!(f, "v {}", deg),
            #[cfg(feature = "roland")]
            Cmd::Buzzer(pw) => write!(f, "b {}", pw),
            #[cfg(feature = "roland")]
            Cmd::TrackSensor => write!(f, "t"),
            #[cfg(feature = "roland")]
            Cmd::GetPosition => write!(f, "P"),

            Cmd::SetPin(pin, value) => write!(f, "p {} {}", pin, *value as u8),
            Cmd::SetPwm(pin, hz, cycle) => write!(f, "w {} {} {}", pin, hz, cycle),
            Cmd::ServoBasic(pin, deg) => write!(f, "V {} {}", pin, deg),
            Cmd::GetTime => write!(f, "z"),
        }
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
macro_rules! parse_bool {
    ($b:expr) => {
        if $b == 1 {
            true
        } else if $b == 0 {
            false
        } else {
            Err(anyhow!(
                "invalid arg: {}, can be 1 for high or 0 for low",
                $b
            ))?
        }
    };
}

impl FromStr for Cmd {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.split_whitespace().peekable();

        let c = iter.next().ok_or_else(|| anyhow!("missing command"))?;
        let args = iter.collect::<Vec<_>>();

        let res = match c {
            #[cfg(feature = "roland")]
            "m" => {
                let x = parse!(args 2);
                Cmd::MoveRobot(x[0], x[1])
            }
            "M" => {
                if args.len() < 3 {
                    Err(anyhow!("Didn't provide angle, speed",))?
                }

                let angle = args[0].parse::<f64>()?;
                let speed = args[1].parse::<i8>()?;

                Cmd::MoveRobotByAngle(angle, speed)
            }
            #[cfg(feature = "roland")]
            "s" => Cmd::StopRobot,
            #[cfg(feature = "roland")]
            "l" => {
                let x: Vec<u8> = parse!(args 3);
                Cmd::Led(parse_bool!(x[0]), parse_bool!(x[1]), parse_bool!(x[2]))
            }
            #[cfg(feature = "roland")]
            "v" => {
                let x = parse!(args 1);
                Cmd::ServoAbsolute(x[0])
            }
            #[cfg(feature = "roland")]
            "t" => Cmd::TrackSensor,
            #[cfg(feature = "roland")]
            "b" => {
                let x = parse!(args 1);
                Cmd::Buzzer(x[0])
            }
            #[cfg(feature = "roland")]
            "P" => Cmd::GetPosition,

            "p" => {
                let x = parse!(args 2);
                Cmd::SetPin(x[0], parse_bool!(x[1]))
            }
            "w" => {
                let x = parse!(args 3);
                Cmd::SetPwm(x[0] as u8, x[1], x[2])
            }
            "V" => {
                let x: Vec<i16> = parse!(args 2);
                Cmd::ServoBasic(x[0] as u8, x[1] as i8)
            }
            "z" => Cmd::GetTime,

            _ => Err(anyhow!("invalid command"))?,
        };
        Ok(res)
    }
}

pub type SensorData = [bool; 4];
// parse incoming data for the client
pub fn parse_track_sensor_data(s: &str) -> Result<SensorData> {
    let v = s
        .split(',')
        .map(|s| {
            if s == "1" {
                Ok(true)
            } else if s == "0" {
                Ok(false)
            } else {
                Err(anyhow!("invalid number"))
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    let len = v.len();
    match v.try_into() {
        Ok(d) => Ok(d),
        Err(_) => Err(anyhow!("Expected a Vec of length {} but it was {len}", 4))?,
    }
}

pub fn parse_position_data(s: &str) -> Result<Option<Position>> {
    if s.is_empty() {
        return Ok(None);
    }

    let v: Result<Vec<f64>, _> = s.split(',').map(|s| s.parse::<f64>()).collect();
    match v {
        Ok(vec) if vec.len() == 3 => Ok(Some(Position::new(vec[0], vec[1], vec[2]))),
        _ => Err(anyhow!("Expected three floats")),
    }
}

pub fn get_time() -> Result<f64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros() as f64 / 1000.0)?)
}

mod tests {
    #![allow(unused_imports)]
    use std::str::FromStr;

    #[test]
    fn parse_sensor_data() {
        let s = "1,0,1,0";
        let res = super::parse_track_sensor_data(s);
        assert_eq!(res.unwrap(), [true, false, true, false]);
    }

    #[test]
    fn parse_sensor_data_err() {
        for s in ["", " ", "1,1,1", "0,0,0,", "1,h,0,1", "1,0,1,1,1"] {
            let res = super::parse_track_sensor_data(s);
            dbg!(&res);
            assert!(res.is_err());
        }
    }

    #[test]
    fn get_time() -> anyhow::Result<()> {
        let t = super::get_time()?;
        assert!(t > 0.0);
        Ok(())
    }

    #[test]
    fn cmd_from_str() {
        let s = "p 1 0";
        let res = super::Cmd::from_str(s);
        assert!(res.is_ok());
        let cmd = res.unwrap();
        assert_eq!(cmd, super::Cmd::SetPin(1, false));
    }

    #[test]
    fn cmd_from_str_err() {
        let s = "m 1";
        let res = super::Cmd::from_str(s);
        assert!(res.is_err());

        assert!(super::Cmd::from_str("").is_err());
        assert!(super::Cmd::from_str(" ").is_err());
        assert!(super::Cmd::from_str("p 1 2").is_err());
    }

    #[test]
    fn cmd_to_string() {
        let cmd = super::Cmd::SetPwm(1, 6.9, 4.20);
        assert_eq!(cmd.to_string(), "w 1 6.9 4.2");
    }

    #[test]
    fn str_to_str() {
        assert_eq!(super::Cmd::from_str("m 1 2").unwrap().to_string(), "m 1 2");
    }

    #[test]
    fn cmd_to_cmd() {
        let cmd = super::Cmd::SetPwm(1, 6.9, 4.20);
        let cmd2 = super::Cmd::from_str(cmd.to_string().as_str()).unwrap();
        assert_eq!(cmd, cmd2);
    }
}
