use std::{mem::size_of, time::Duration};

use crate::cmd::SEPARATOR;

pub trait Readable
where
    Self: Sized,
{
    fn parse_str<'a>(s: &mut impl Iterator<Item = &'a str>) -> anyhow::Result<Self>;
    fn parse_binary(r: &mut impl std::io::Read) -> anyhow::Result<Self>;
}
pub trait Writable {
    fn write_str(&self, f: &mut dyn std::fmt::Write) -> std::fmt::Result;
    fn write_binary(&self, w: &mut dyn std::io::Write) -> anyhow::Result<()>;
}

pub mod commands {
    use crate::cmd::{self, Command};

    use super::{Readable, Writable};

    pub enum Concrete {
        #[cfg(feature = "roland")]
        MoveRobot(cmd::MoveRobot),
        #[cfg(feature = "roland")]
        MoveRobotByAngle(cmd::MoveRobotByAngle),
        #[cfg(feature = "roland")]
        StopRobot(cmd::StopRobot),
        #[cfg(feature = "roland")]
        Led(cmd::Led),
        #[cfg(feature = "roland")]
        ServoAbsolute(cmd::ServoAbsolute),
        #[cfg(feature = "roland")]
        Buzzer(cmd::Buzzer),
        #[cfg(feature = "roland")]
        TrackSensor(cmd::TrackSensor),
        #[cfg(feature = "roland")]
        UltraSensor(cmd::UltraSensor),

        #[cfg(feature = "gpio")]
        ReadPin(cmd::ReadPin),
        #[cfg(feature = "gpio")]
        SetPin(cmd::SetPin),
        #[cfg(feature = "gpio")]
        SetPwm(cmd::SetPwm),
        #[cfg(feature = "gpio")]
        ServoBasic(cmd::ServoBasic),

        #[cfg(feature = "camloc")]
        GetPosition(cmd::GetPosition),

        Nop(cmd::Nop),
        GetUptime(cmd::GetUptime),
    }

    impl Concrete {
        pub fn parse_str<'a>(s: &mut impl Iterator<Item = &'a str>) -> anyhow::Result<Self> {
            let Some(prefix) = s.next() else {
                return Err(anyhow::Error::msg("Missing prefix"));
            };
            let Some(prefix) = prefix.chars().next() else {
                return Err(anyhow::Error::msg("Prefix can't be empty"));
            };

            Ok(match prefix {
                #[cfg(feature = "roland")]
                cmd::MoveRobot::PREFIX => Self::MoveRobot(cmd::MoveRobot::parse_str(s)?),
                #[cfg(feature = "roland")]
                cmd::MoveRobotByAngle::PREFIX => {
                    Self::MoveRobotByAngle(cmd::MoveRobotByAngle::parse_str(s)?)
                }
                #[cfg(feature = "roland")]
                cmd::StopRobot::PREFIX => Self::StopRobot(cmd::StopRobot::parse_str(s)?),
                #[cfg(feature = "roland")]
                cmd::Led::PREFIX => Self::Led(cmd::Led::parse_str(s)?),
                #[cfg(feature = "roland")]
                cmd::ServoAbsolute::PREFIX => {
                    Self::ServoAbsolute(cmd::ServoAbsolute::parse_str(s)?)
                }
                #[cfg(feature = "roland")]
                cmd::Buzzer::PREFIX => Self::Buzzer(cmd::Buzzer::parse_str(s)?),
                #[cfg(feature = "roland")]
                cmd::TrackSensor::PREFIX => Self::TrackSensor(cmd::TrackSensor::parse_str(s)?),
                #[cfg(feature = "roland")]
                cmd::UltraSensor::PREFIX => Self::UltraSensor(cmd::UltraSensor::parse_str(s)?),

                #[cfg(feature = "gpio")]
                cmd::ReadPin::PREFIX => Self::ReadPin(cmd::ReadPin::parse_str(s)?),
                #[cfg(feature = "gpio")]
                cmd::SetPin::PREFIX => Self::SetPin(cmd::SetPin::parse_str(s)?),
                #[cfg(feature = "gpio")]
                cmd::SetPwm::PREFIX => Self::SetPwm(cmd::SetPwm::parse_str(s)?),
                #[cfg(feature = "gpio")]
                cmd::ServoBasic::PREFIX => Self::ServoBasic(cmd::ServoBasic::parse_str(s)?),

                #[cfg(feature = "camloc")]
                cmd::GetPosition::PREFIX => Self::GetPosition(cmd::GetPosition::parse_str(s)?),

                cmd::Nop::PREFIX => Self::Nop(cmd::Nop::parse_str(s)?),
                cmd::GetUptime::PREFIX => Self::GetUptime(cmd::GetUptime::parse_str(s)?),

                _ => return Err(anyhow::Error::msg("Unknown prefix")),
            })
        }
        pub fn parse_binary(r: &mut impl std::io::Read) -> anyhow::Result<Self> {
            let mut prefix = [0; std::mem::size_of::<char>()];
            r.read_exact(&mut prefix)?;
            let Some(prefix) = char::from_u32(u32::from_be_bytes(prefix)) else {
                return Err(anyhow::Error::msg("Invalid prefix"));
            };

            Ok(match prefix {
                #[cfg(feature = "roland")]
                cmd::MoveRobot::PREFIX => Self::MoveRobot(cmd::MoveRobot::parse_binary(r)?),
                #[cfg(feature = "roland")]
                cmd::MoveRobotByAngle::PREFIX => {
                    Self::MoveRobotByAngle(cmd::MoveRobotByAngle::parse_binary(r)?)
                }
                #[cfg(feature = "roland")]
                cmd::StopRobot::PREFIX => Self::StopRobot(cmd::StopRobot::parse_binary(r)?),
                #[cfg(feature = "roland")]
                cmd::Led::PREFIX => Self::Led(cmd::Led::parse_binary(r)?),
                #[cfg(feature = "roland")]
                cmd::ServoAbsolute::PREFIX => {
                    Self::ServoAbsolute(cmd::ServoAbsolute::parse_binary(r)?)
                }
                #[cfg(feature = "roland")]
                cmd::Buzzer::PREFIX => Self::Buzzer(cmd::Buzzer::parse_binary(r)?),
                #[cfg(feature = "roland")]
                cmd::TrackSensor::PREFIX => Self::TrackSensor(cmd::TrackSensor::parse_binary(r)?),
                #[cfg(feature = "roland")]
                cmd::UltraSensor::PREFIX => Self::UltraSensor(cmd::UltraSensor::parse_binary(r)?),

                #[cfg(feature = "gpio")]
                cmd::ReadPin::PREFIX => Self::ReadPin(cmd::ReadPin::parse_binary(r)?),
                #[cfg(feature = "gpio")]
                cmd::SetPin::PREFIX => Self::SetPin(cmd::SetPin::parse_binary(r)?),
                #[cfg(feature = "gpio")]
                cmd::SetPwm::PREFIX => Self::SetPwm(cmd::SetPwm::parse_binary(r)?),
                #[cfg(feature = "gpio")]
                cmd::ServoBasic::PREFIX => Self::ServoBasic(cmd::ServoBasic::parse_binary(r)?),

                #[cfg(feature = "camloc")]
                cmd::GetPosition::PREFIX => Self::GetPosition(cmd::GetPosition::parse_binary(r)?),

                cmd::Nop::PREFIX => Self::Nop(cmd::Nop::parse_binary(r)?),
                cmd::GetUptime::PREFIX => Self::GetUptime(cmd::GetUptime::parse_binary(r)?),

                _ => return Err(anyhow::Error::msg("Unknown prefix")),
            })
        }

        pub fn write_str(&self, f: &mut dyn std::fmt::Write) -> std::fmt::Result {
            match self {
                #[cfg(feature = "roland")]
                Self::MoveRobot(c) => {
                    f.write_char(cmd::MoveRobot::PREFIX)?;
                    c.write_str(f)?;
                }
                #[cfg(feature = "roland")]
                Self::MoveRobotByAngle(c) => {
                    f.write_char(cmd::MoveRobotByAngle::PREFIX)?;
                    c.write_str(f)?;
                }
                #[cfg(feature = "roland")]
                Self::StopRobot(c) => {
                    f.write_char(cmd::StopRobot::PREFIX)?;
                    c.write_str(f)?;
                }
                #[cfg(feature = "roland")]
                Self::Led(c) => {
                    f.write_char(cmd::Led::PREFIX)?;
                    c.write_str(f)?;
                }
                #[cfg(feature = "roland")]
                Self::ServoAbsolute(c) => {
                    f.write_char(cmd::ServoAbsolute::PREFIX)?;
                    c.write_str(f)?;
                }
                #[cfg(feature = "roland")]
                Self::Buzzer(c) => {
                    f.write_char(cmd::Buzzer::PREFIX)?;
                    c.write_str(f)?;
                }
                #[cfg(feature = "roland")]
                Self::TrackSensor(c) => {
                    f.write_char(cmd::TrackSensor::PREFIX)?;
                    c.write_str(f)?;
                }
                #[cfg(feature = "roland")]
                Self::UltraSensor(c) => {
                    f.write_char(cmd::UltraSensor::PREFIX)?;
                    c.write_str(f)?;
                }

                #[cfg(feature = "gpio")]
                Self::ReadPin(c) => {
                    f.write_char(cmd::ReadPin::PREFIX)?;
                    c.write_str(f)?;
                }
                #[cfg(feature = "gpio")]
                Self::SetPin(c) => {
                    f.write_char(cmd::SetPin::PREFIX)?;
                    c.write_str(f)?;
                }
                #[cfg(feature = "gpio")]
                Self::SetPwm(c) => {
                    f.write_char(cmd::SetPwm::PREFIX)?;
                    c.write_str(f)?;
                }
                #[cfg(feature = "gpio")]
                Self::ServoBasic(c) => {
                    f.write_char(cmd::ServoBasic::PREFIX)?;
                    c.write_str(f)?;
                }

                #[cfg(feature = "camloc")]
                Self::GetPosition(c) => {
                    f.write_char(cmd::GetPosition::PREFIX)?;
                    c.write_str(f)?;
                }

                Self::Nop(c) => {
                    f.write_char(cmd::Nop::PREFIX)?;
                    c.write_str(f)?;
                }
                Self::GetUptime(c) => {
                    f.write_char(cmd::GetUptime::PREFIX)?;
                    c.write_str(f)?;
                }
            }
            Ok(())
        }
        pub fn write_binary(&self, w: &mut dyn std::io::Write) -> anyhow::Result<()> {
            match self {
                #[cfg(feature = "roland")]
                Self::MoveRobot(c) => {
                    w.write_all(&(cmd::MoveRobot::PREFIX as u32).to_be_bytes())?;
                    c.write_binary(w)?;
                }
                #[cfg(feature = "roland")]
                Self::MoveRobotByAngle(c) => {
                    w.write_all(&(cmd::MoveRobotByAngle::PREFIX as u32).to_be_bytes())?;
                    c.write_binary(w)?;
                }
                #[cfg(feature = "roland")]
                Self::StopRobot(c) => {
                    w.write_all(&(cmd::StopRobot::PREFIX as u32).to_be_bytes())?;
                    c.write_binary(w)?;
                }
                #[cfg(feature = "roland")]
                Self::Led(c) => {
                    w.write_all(&(cmd::Led::PREFIX as u32).to_be_bytes())?;
                    c.write_binary(w)?;
                }
                #[cfg(feature = "roland")]
                Self::ServoAbsolute(c) => {
                    w.write_all(&(cmd::ServoAbsolute::PREFIX as u32).to_be_bytes())?;
                    c.write_binary(w)?;
                }
                #[cfg(feature = "roland")]
                Self::Buzzer(c) => {
                    w.write_all(&(cmd::Buzzer::PREFIX as u32).to_be_bytes())?;
                    c.write_binary(w)?;
                }
                #[cfg(feature = "roland")]
                Self::TrackSensor(c) => {
                    w.write_all(&(cmd::TrackSensor::PREFIX as u32).to_be_bytes())?;
                    c.write_binary(w)?;
                }
                #[cfg(feature = "roland")]
                Self::UltraSensor(c) => {
                    w.write_all(&(cmd::UltraSensor::PREFIX as u32).to_be_bytes())?;
                    c.write_binary(w)?;
                }

                #[cfg(feature = "gpio")]
                Self::ReadPin(c) => {
                    w.write_all(&(cmd::ReadPin::PREFIX as u32).to_be_bytes())?;
                    c.write_binary(w)?;
                }
                #[cfg(feature = "gpio")]
                Self::SetPin(c) => {
                    w.write_all(&(cmd::SetPin::PREFIX as u32).to_be_bytes())?;
                    c.write_binary(w)?;
                }
                #[cfg(feature = "gpio")]
                Self::SetPwm(c) => {
                    w.write_all(&(cmd::SetPwm::PREFIX as u32).to_be_bytes())?;
                    c.write_binary(w)?;
                }
                #[cfg(feature = "gpio")]
                Self::ServoBasic(c) => {
                    w.write_all(&(cmd::ServoBasic::PREFIX as u32).to_be_bytes())?;
                    c.write_binary(w)?;
                }

                #[cfg(feature = "camloc")]
                Self::GetPosition(c) => {
                    w.write_all(&(cmd::GetPosition::PREFIX as u32).to_be_bytes())?;
                    c.write_binary(w)?;
                }

                Self::Nop(c) => {
                    w.write_all(&(cmd::Nop::PREFIX as u32).to_be_bytes())?;
                    c.write_binary(w)?;
                }
                Self::GetUptime(c) => {
                    w.write_all(&(cmd::GetUptime::PREFIX as u32).to_be_bytes())?;
                    c.write_binary(w)?;
                }
            }
            Ok(())
        }
    }
}

#[cfg(feature = "camloc")]
impl Readable for crate::camloc::Position {
    fn parse_str<'a>(s: &mut impl Iterator<Item = &'a str>) -> anyhow::Result<Self> {
        use anyhow::Error;
        Ok(Self {
            x: s.next()
                .ok_or_else(|| Error::msg("No x"))?
                .parse()
                .map_err(|_| Error::msg("Couldn't parse x"))?,
            y: s.next()
                .ok_or_else(|| Error::msg("No y"))?
                .parse()
                .map_err(|_| Error::msg("Couldn't parse y"))?,
            rotation: s
                .next()
                .ok_or_else(|| Error::msg("No rotation"))?
                .parse()
                .map_err(|_| Error::msg("Couldn't parse rotation"))?,
        })
    }

    fn parse_binary(r: &mut impl std::io::Read) -> anyhow::Result<Self> {
        let mut f = [0; 8];
        r.read_exact(&mut f)?;
        let x = f64::from_be_bytes(f);
        let mut f = [0; 8];
        r.read_exact(&mut f)?;
        let y = f64::from_be_bytes(f);
        let mut f = [0; 8];
        r.read_exact(&mut f)?;
        let rotation = f64::from_be_bytes(f);
        Ok(Self { x, y, rotation })
    }
}
#[cfg(feature = "camloc")]
impl Writable for crate::camloc::Position {
    fn write_str(&self, f: &mut dyn std::fmt::Write) -> std::fmt::Result {
        write!(f, "{}", self.x)?;
        write!(f, "{}", self.y)?;
        write!(f, "{}", self.rotation)
    }

    fn write_binary(&self, w: &mut dyn std::io::Write) -> anyhow::Result<()> {
        w.write_all(&self.x.to_be_bytes())?;
        w.write_all(&self.y.to_be_bytes())?;
        w.write_all(&self.rotation.to_be_bytes())?;

        Ok(())
    }
}

impl Readable for () {
    fn parse_str<'a>(_: &mut impl Iterator<Item = &'a str>) -> anyhow::Result<Self> {
        Ok(())
    }
    fn parse_binary(_: &mut impl std::io::Read) -> anyhow::Result<Self> {
        Ok(())
    }
}
impl Writable for () {
    fn write_str(&self, _: &mut dyn std::fmt::Write) -> std::fmt::Result {
        Ok(())
    }

    fn write_binary(&self, _: &mut dyn std::io::Write) -> anyhow::Result<()> {
        Ok(())
    }
}

impl Readable for Duration {
    fn parse_str<'a>(s: &mut impl Iterator<Item = &'a str>) -> anyhow::Result<Self> {
        let Some(s) = s.next() else {
            return Err(anyhow::Error::msg("Not enough arguments"))
        };

        Ok(Duration::from_millis(s.parse::<u64>().map_err(|_| {
            anyhow::Error::msg("Couldn't parse millis")
        })?))
    }
    fn parse_binary(r: &mut impl std::io::Read) -> anyhow::Result<Self> {
        let mut millis = [0; std::mem::size_of::<u64>()];
        r.read_exact(&mut millis)?;
        Ok(Duration::from_millis(u64::from_be_bytes(millis)))
    }
}
impl Writable for Duration {
    fn write_str(&self, f: &mut dyn std::fmt::Write) -> std::fmt::Result {
        write!(f, "{}", self.as_millis())
    }
    fn write_binary(&self, w: &mut dyn std::io::Write) -> anyhow::Result<()> {
        w.write_all(&self.as_millis().to_be_bytes())?;
        Ok(())
    }
}

impl<T: Readable> Readable for Option<T> {
    fn parse_str<'a>(s: &mut impl Iterator<Item = &'a str>) -> anyhow::Result<Self> {
        let Some(v) = s.next() else {
            return Err(anyhow::Error::msg("Not enough arguments"))
        };

        match v {
            "0" => Ok(None),
            "1" => Ok(Some(T::parse_str(s)?)),
            _ => Err(anyhow::Error::msg(
                "Option variant needs to be indicated with a 0 or a 1",
            )),
        }
    }

    fn parse_binary(r: &mut impl std::io::Read) -> anyhow::Result<Self> {
        let mut v = [0; std::mem::size_of::<u8>()];
        r.read_exact(&mut v)?;

        if u8::from_be_bytes(v) == 0 {
            return Ok(None);
        }

        Ok(Some(T::parse_binary(r)?))
    }
}
impl<T: Writable> Writable for Option<T> {
    fn write_str(&self, f: &mut dyn std::fmt::Write) -> std::fmt::Result {
        if let Some(v) = self {
            write!(f, "1{}", SEPARATOR)?;
            v.write_str(f)?;
        } else {
            write!(f, "0")?;
        }
        Ok(())
    }

    fn write_binary(&self, w: &mut dyn std::io::Write) -> anyhow::Result<()> {
        if let Some(v) = self {
            w.write_all(&[1])?;
            v.write_binary(w)?;
        } else {
            w.write_all(&[0])?;
        }
        Ok(())
    }
}

impl Readable for bool {
    fn parse_str<'a>(s: &mut impl Iterator<Item = &'a str>) -> anyhow::Result<Self> {
        let Some(s) = s.next() else {
            return Err(anyhow::Error::msg("Not enough arguments"))
        };

        match s {
            "0" => Ok(false),
            "1" => Ok(true),
            _ => Err(anyhow::Error::msg("Couldn't parse bool")),
        }
    }

    fn parse_binary(r: &mut impl std::io::Read) -> anyhow::Result<Self> {
        let mut b = [0];
        r.read_exact(&mut b)?;
        Ok(u8::from_be_bytes(b) != 0)
    }
}
impl Writable for bool {
    fn write_str(&self, f: &mut dyn std::fmt::Write) -> std::fmt::Result {
        write!(f, "{}", *self as u8)
    }

    fn write_binary(&self, w: &mut dyn std::io::Write) -> anyhow::Result<()> {
        w.write_all(&(*self as u8).to_be_bytes())?;
        Ok(())
    }
}

#[cfg(feature = "camloc")]
impl Readable for crate::camloc::MotionHint {
    fn parse_str<'a>(s: &mut impl Iterator<Item = &'a str>) -> anyhow::Result<Self> {
        let Some(s) = s.next() else {
            return Err(anyhow::Error::msg("Not enough arguments"))
        };

        match s {
            "f" => Ok(Self::MovingForwards),
            "b" => Ok(Self::MovingBackwards),
            "s" => Ok(Self::Stationary),
            _ => Err(anyhow::Error::msg("Couldn't parse motion hint")),
        }
    }

    fn parse_binary(r: &mut impl std::io::Read) -> anyhow::Result<Self> {
        let mut c = [0];
        r.read_exact(&mut c)?;
        match u8::from_be_bytes(c) as char {
            'f' => Ok(Self::MovingForwards),
            'b' => Ok(Self::MovingBackwards),
            's' => Ok(Self::Stationary),
            _ => Err(anyhow::Error::msg("Couldn't parse motion hint")),
        }
    }
}
#[cfg(feature = "camloc")]
impl Writable for crate::camloc::MotionHint {
    fn write_str(&self, f: &mut dyn std::fmt::Write) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::MovingForwards => 'f',
                Self::MovingBackwards => 'b',
                Self::Stationary => 's',
            }
        )
    }

    fn write_binary(&self, w: &mut dyn std::io::Write) -> anyhow::Result<()> {
        let c = match self {
            Self::MovingForwards => 'f',
            Self::MovingBackwards => 'b',
            Self::Stationary => 's',
        };
        w.write_all(&(c as u32).to_be_bytes())?;
        Ok(())
    }
}

impl<T: Readable, const N: usize> Readable for [T; N] {
    fn parse_str<'a>(s: &mut impl Iterator<Item = &'a str>) -> anyhow::Result<Self> {
        let Some(n) = s.next() else {
            return Err(anyhow::Error::msg("Missing array length"));
        };
        let Ok(n) = n.parse::<u64>() else {
            return Err(anyhow::Error::msg("Couldn't parse array length"));
        };
        if n as usize != N {
            return Err(anyhow::Error::msg("Array length doesn't match"));
        }

        let mut v = vec![];
        for _ in 0..N {
            v.push(T::parse_str(s)?);
        }

        // because no debug...
        Ok(unsafe { v.try_into().unwrap_unchecked() })
    }

    fn parse_binary(r: &mut impl std::io::Read) -> anyhow::Result<Self> {
        let mut n = [0; size_of::<u64>()];
        r.read_exact(&mut n)?;
        let n = u64::from_be_bytes(n);

        if n as usize != N {
            return Err(anyhow::Error::msg("Array length doesn't match"));
        }

        let mut v = vec![];
        for _ in 0..N {
            v.push(T::parse_binary(r)?);
        }

        // because no debug...
        Ok(unsafe { v.try_into().unwrap_unchecked() })
    }
}
impl<T: Writable, const N: usize> Writable for [T; N] {
    fn write_str(&self, f: &mut dyn std::fmt::Write) -> std::fmt::Result {
        write!(f, "{}", N as u64)?;
        for t in self {
            write!(f, "{}", SEPARATOR)?;
            t.write_str(f)?;
        }
        Ok(())
    }

    fn write_binary(&self, w: &mut dyn std::io::Write) -> anyhow::Result<()> {
        w.write_all(&(N as u64).to_be_bytes())?;
        for t in self {
            t.write_binary(w)?;
        }
        Ok(())
    }
}

macro_rules! primitve_impl {
    ($t:tt) => {
        impl Readable for $t {
            fn parse_str<'a>(s: &mut impl Iterator<Item = &'a str>) -> anyhow::Result<Self> {
                let Some(s) = s.next() else {
                                            return Err(anyhow::Error::msg("Not enough arguments"))
                                        };

                s.parse()
                    .map_err(|_| anyhow::Error::msg("Couldn't parse millis"))
            }
            fn parse_binary(r: &mut impl std::io::Read) -> anyhow::Result<Self> {
                let mut buf = [0; std::mem::size_of::<Self>()];
                r.read_exact(&mut buf)?;
                Ok(Self::from_be_bytes(buf))
            }
        }
        impl Writable for $t {
            fn write_str(&self, f: &mut dyn std::fmt::Write) -> std::fmt::Result {
                write!(f, "{}", self)
            }

            fn write_binary(&self, w: &mut dyn std::io::Write) -> anyhow::Result<()> {
                w.write_all(&self.to_be_bytes())?;
                Ok(())
            }
        }
    };
}

primitve_impl!(u8);
primitve_impl!(u16);
primitve_impl!(u32);
primitve_impl!(u64);
primitve_impl!(u128);

primitve_impl!(i8);
primitve_impl!(i16);
primitve_impl!(i32);
primitve_impl!(i64);
primitve_impl!(i128);

primitve_impl!(f32);
primitve_impl!(f64);

// TODO: tuples
