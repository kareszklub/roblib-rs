#[cfg(feature = "roland")]
use crate::roland::Roland;

use crate::Robot;

use anyhow::{anyhow, Result};

use std::{
    fmt::Display,
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

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
    /// P
    #[cfg(feature = "camloc")]
    GetPosition,

    /// p
    #[cfg(feature = "gpio")]
    SetPin(u8, bool),
    /// w
    #[cfg(feature = "gpio")]
    SetPwm(u8, f64, f64),
    /// V
    #[cfg(feature = "gpio")]
    ServoBasic(u8, f64),

    /// z
    GetTime,
}

impl Cmd {
    #[allow(unused_variables)]
    pub async fn exec(&self, robot: &Robot) -> anyhow::Result<Option<String>> {
        let res = match self {
            #[cfg(feature = "roland")]
            Cmd::MoveRobot(left, right) => {
                debug!("Moving robot: {left}:{right}");

                if let Some(r) = &robot.roland {
                    #[allow(clippy::let_unit_value)]
                    let hint = r.drive(*left, *right)?;
                    #[cfg(feature = "camloc")]
                    if let Some(r) = &robot.camloc_service {
                        r.set_motion_hint(hint).await;
                    }
                }

                None
            }

            #[cfg(feature = "roland")]
            Cmd::MoveRobotByAngle(angle, speed) => {
                debug!("Moving robot by angle: {}:{}", angle, speed);

                if let Some(r) = &robot.roland {
                    #[allow(clippy::let_unit_value)]
                    let hint = r.drive_by_angle(*angle, *speed)?;
                    #[cfg(feature = "camloc")]
                    if let Some(r) = &robot.camloc_service {
                        r.set_motion_hint(hint).await;
                    }
                }
                None
            }

            #[cfg(feature = "roland")]
            Cmd::StopRobot => {
                debug!("Stopping robot");

                if let Some(r) = &robot.roland {
                    r.drive(0., 0.)?;
                }
                None
            }

            #[cfg(feature = "roland")]
            Cmd::Led(r, g, b) => {
                debug!("LED: {r}:{g}:{b}");

                if let Some(rr) = &robot.roland {
                    rr.led(*r, *g, *b)?;
                }
                None
            }

            #[cfg(feature = "roland")]
            Cmd::ServoAbsolute(deg) => {
                debug!("Servo absolute: {deg}");

                if let Some(r) = &robot.roland {
                    r.servo(*deg)?;
                }
                None
            }

            #[cfg(feature = "roland")]
            Cmd::Buzzer(pw) => {
                debug!("Buzzer: {pw}");

                if let Some(r) = &robot.roland {
                    r.buzzer(*pw)?
                }
                None
            }

            #[cfg(feature = "roland")]
            Cmd::TrackSensor => {
                debug!("Track sensor");

                let res = if let Some(r) = &robot.roland {
                    r.track_sensor()?
                } else {
                    [false, false, false, false]
                }
                .map(|b| b as u8);

                Some(format!("{},{},{},{}", res[0], res[1], res[2], res[3]))
            }

            #[cfg(feature = "roland")]
            Cmd::UltraSensor => {
                debug!("Ultra sensor");

                let res = if let Some(r) = &robot.roland {
                    r.ultra_sensor()?
                } else {
                    f64::NAN
                };

                Some(format!("{}", res))
            }

            #[cfg(feature = "camloc")]
            Cmd::GetPosition => {
                debug!("Get position");

                Some(if let Some(pos) = robot.get_position().await? {
                    format!("{},{},{}", pos.x, pos.y, pos.rotation)
                } else {
                    String::new()
                })
            }

            #[cfg(feature = "gpio")]
            Cmd::SetPin(pin, value) => {
                debug!("Set pin: {pin}:{value}");

                if let Some(r) = &robot.raw_gpio {
                    r.set(*pin, *value)?;
                }
                None
            }

            #[cfg(feature = "gpio")]
            Cmd::SetPwm(pin, hz, cycle) => {
                debug!("Set pwm: {pin}:{hz}:{cycle}");

                if let Some(r) = &robot.raw_gpio {
                    r.pwm(*pin, *hz, *cycle)?;
                }
                None
            }

            #[cfg(feature = "gpio")]
            Cmd::ServoBasic(pin, deg) => {
                debug!("Servo basic: {deg}");

                if let Some(r) = &robot.raw_gpio {
                    r.servo(*pin, *deg)?;
                }

                None
            }

            Cmd::GetTime => Some(format!("{:.3}", get_time()?)),
        };
        Ok(res)
    }

    pub async fn exec_str(s: &str, robot: &Robot) -> String {
        let cmd = Cmd::from_str(s);
        match cmd {
            Ok(cmd) => match cmd.exec(robot).await {
                Ok(opt) => opt.unwrap_or_else(|| "OK".into()),
                Err(e) => e.to_string(),
            },
            Err(e) => e.to_string(),
        }
    }
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
            Cmd::SetPin(pin, value) => write!(f, "p {} {}", pin, *value as u8),

            #[cfg(feature = "gpio")]
            Cmd::SetPwm(pin, hz, cycle) => write!(f, "w {pin} {hz} {cycle}"),

            #[cfg(feature = "gpio")]
            Cmd::ServoBasic(pin, deg) => write!(f, "V {pin} {deg}"),

            #[cfg(feature = "camloc")]
            Cmd::GetPosition => write!(f, "P"),

            Cmd::GetTime => write!(f, "z"),
        }
    }
}

#[allow(unused)]
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

#[allow(unused)]
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

        #[allow(unused)]
        let args: Vec<&str> = iter.collect();

        let res = match c {
            #[cfg(feature = "roland")]
            "m" => {
                let x = parse!(args 2);
                Cmd::MoveRobot(x[0], x[1])
            }

            #[cfg(feature = "roland")]
            "M" => {
                let x = parse!(args 2);
                Cmd::MoveRobotByAngle(x[0], x[1])
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
            "u" => Cmd::UltraSensor,

            #[cfg(feature = "gpio")]
            "p" => {
                let x = parse!(args 2);
                Cmd::SetPin(x[0], parse_bool!(x[1]))
            }

            #[cfg(feature = "gpio")]
            "w" => {
                let x = parse!(args 3);
                Cmd::SetPwm(x[0] as u8, x[1], x[2])
            }

            #[cfg(feature = "gpio")]
            "V" => {
                let x: Vec<f64> = parse!(args 2);
                Cmd::ServoBasic(x[0] as u8, x[1])
            }

            #[cfg(feature = "camloc")]
            "P" => Cmd::GetPosition,

            "z" => Cmd::GetTime,

            _ => Err(anyhow!("invalid command"))?,
        };

        Ok(res)
    }
}

#[cfg(feature = "roland")]
pub fn parse_track_sensor_data(s: &str) -> Result<[bool; 4]> {
    println!("parsing track sensor data '{s}'");
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
        Err(_) => Err(anyhow!("Expected a Vec of length 4 but it was {}", len))?,
    }
}

#[cfg(feature = "camloc")]
pub fn parse_position_data(s: &str) -> Result<Option<camloc_server::Position>> {
    if s.is_empty() {
        return Ok(None);
    }

    let v: Result<Vec<f64>, _> = s.split(',').map(|s| s.parse::<f64>()).collect();
    match v {
        Ok(vec) if vec.len() == 3 => Ok(Some(camloc_server::Position::new(vec[0], vec[1], vec[2]))),
        _ => Err(anyhow!("Expected three floats")),
    }
}

#[cfg(feature = "roland")]
pub fn parse_ultra_sensor_data(s: &str) -> Result<f64> {
    s.parse().map_err(|_| anyhow!("Expected a float"))
}

pub fn get_time() -> Result<f64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros() as f64 / 1000.0)?)
}
