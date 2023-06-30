use std::fmt::Display;

use crate::cmd::{self, Command};

use roblib_parsing::{Readable, Writable, SEPARATOR};

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

    Subscribe(cmd::Subscribe),
    Unsubscribe(cmd::Unsubscribe),

    Nop(cmd::Nop),
    GetUptime(cmd::GetUptime),
}

// TODO: automatize Concrete impls

#[cfg_attr(feature = "async", async_trait::async_trait)]
impl Readable for Concrete {
    fn parse_text<'a>(s: &mut impl Iterator<Item = &'a str>) -> anyhow::Result<Self> {
        let Some(prefix) = s.next() else {
            return Err(anyhow::Error::msg("Missing prefix"));
        };
        let Some(prefix) = prefix.chars().next() else {
            return Err(anyhow::Error::msg("Prefix can't be empty"));
        };

        Ok(match prefix {
            #[cfg(feature = "roland")]
            cmd::MoveRobot::PREFIX => Self::MoveRobot(cmd::MoveRobot::parse_text(s)?),
            #[cfg(feature = "roland")]
            cmd::MoveRobotByAngle::PREFIX => {
                Self::MoveRobotByAngle(cmd::MoveRobotByAngle::parse_text(s)?)
            }
            #[cfg(feature = "roland")]
            cmd::StopRobot::PREFIX => Self::StopRobot(cmd::StopRobot::parse_text(s)?),
            #[cfg(feature = "roland")]
            cmd::Led::PREFIX => Self::Led(cmd::Led::parse_text(s)?),
            #[cfg(feature = "roland")]
            cmd::ServoAbsolute::PREFIX => Self::ServoAbsolute(cmd::ServoAbsolute::parse_text(s)?),
            #[cfg(feature = "roland")]
            cmd::Buzzer::PREFIX => Self::Buzzer(cmd::Buzzer::parse_text(s)?),
            #[cfg(feature = "roland")]
            cmd::TrackSensor::PREFIX => Self::TrackSensor(cmd::TrackSensor::parse_text(s)?),
            #[cfg(feature = "roland")]
            cmd::UltraSensor::PREFIX => Self::UltraSensor(cmd::UltraSensor::parse_text(s)?),

            #[cfg(feature = "gpio")]
            cmd::ReadPin::PREFIX => Self::ReadPin(cmd::ReadPin::parse_text(s)?),
            #[cfg(feature = "gpio")]
            cmd::SetPin::PREFIX => Self::SetPin(cmd::SetPin::parse_text(s)?),
            #[cfg(feature = "gpio")]
            cmd::SetPwm::PREFIX => Self::SetPwm(cmd::SetPwm::parse_text(s)?),
            #[cfg(feature = "gpio")]
            cmd::ServoBasic::PREFIX => Self::ServoBasic(cmd::ServoBasic::parse_text(s)?),

            #[cfg(feature = "camloc")]
            cmd::GetPosition::PREFIX => Self::GetPosition(cmd::GetPosition::parse_text(s)?),

            cmd::Nop::PREFIX => Self::Nop(cmd::Nop::parse_text(s)?),
            cmd::GetUptime::PREFIX => Self::GetUptime(cmd::GetUptime::parse_text(s)?),

            _ => return Err(anyhow::anyhow!("Unknown prefix '{prefix}'")),
        })
    }
    fn parse_binary(r: &mut impl std::io::Read) -> anyhow::Result<Self> {
        let mut prefix = [0; 1];
        r.read_exact(&mut prefix)?;
        let prefix = u8::from_be_bytes(prefix) as char;

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
            cmd::ServoAbsolute::PREFIX => Self::ServoAbsolute(cmd::ServoAbsolute::parse_binary(r)?),
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

            _ => return Err(anyhow::anyhow!("Unknown prefix '{prefix}'")),
        })
    }

    #[cfg(feature = "async")]
    async fn parse_binary_async(
        r: &mut (impl ::futures::AsyncRead + Send + Unpin),
    ) -> ::anyhow::Result<Self> {
        use futures::AsyncReadExt;

        let mut prefix = [0; 1];
        r.read_exact(&mut prefix).await?;
        let prefix = u8::from_be_bytes(prefix) as char;

        Ok(match prefix {
            #[cfg(feature = "roland")]
            cmd::MoveRobot::PREFIX => Self::MoveRobot(cmd::MoveRobot::parse_binary_async(r).await?),
            #[cfg(feature = "roland")]
            cmd::MoveRobotByAngle::PREFIX => {
                Self::MoveRobotByAngle(cmd::MoveRobotByAngle::parse_binary_async(r).await?)
            }
            #[cfg(feature = "roland")]
            cmd::StopRobot::PREFIX => Self::StopRobot(cmd::StopRobot::parse_binary_async(r).await?),
            #[cfg(feature = "roland")]
            cmd::Led::PREFIX => Self::Led(cmd::Led::parse_binary_async(r).await?),
            #[cfg(feature = "roland")]
            cmd::ServoAbsolute::PREFIX => {
                Self::ServoAbsolute(cmd::ServoAbsolute::parse_binary_async(r).await?)
            }
            #[cfg(feature = "roland")]
            cmd::Buzzer::PREFIX => Self::Buzzer(cmd::Buzzer::parse_binary_async(r).await?),
            #[cfg(feature = "roland")]
            cmd::TrackSensor::PREFIX => {
                Self::TrackSensor(cmd::TrackSensor::parse_binary_async(r).await?)
            }
            #[cfg(feature = "roland")]
            cmd::UltraSensor::PREFIX => {
                Self::UltraSensor(cmd::UltraSensor::parse_binary_async(r).await?)
            }

            #[cfg(feature = "gpio")]
            cmd::ReadPin::PREFIX => Self::ReadPin(cmd::ReadPin::parse_binary_async(r).await?),
            #[cfg(feature = "gpio")]
            cmd::SetPin::PREFIX => Self::SetPin(cmd::SetPin::parse_binary_async(r).await?),
            #[cfg(feature = "gpio")]
            cmd::SetPwm::PREFIX => Self::SetPwm(cmd::SetPwm::parse_binary_async(r).await?),
            #[cfg(feature = "gpio")]
            cmd::ServoBasic::PREFIX => {
                Self::ServoBasic(cmd::ServoBasic::parse_binary_async(r).await?)
            }

            #[cfg(feature = "camloc")]
            cmd::GetPosition::PREFIX => {
                Self::GetPosition(cmd::GetPosition::parse_binary_async(r).await?)
            }

            cmd::Nop::PREFIX => Self::Nop(cmd::Nop::parse_binary_async(r).await?),
            cmd::GetUptime::PREFIX => Self::GetUptime(cmd::GetUptime::parse_binary_async(r).await?),

            _ => return Err(anyhow::anyhow!("Unknown prefix '{prefix}'")),
        })
    }
}
#[cfg_attr(feature = "async", async_trait::async_trait)]
impl Writable for Concrete {
    fn write_text(&self, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        match self {
            #[cfg(feature = "roland")]
            Self::MoveRobot(c) => {
                write!(w, "{}{}", cmd::MoveRobot::PREFIX, SEPARATOR)?;
                c.write_text(w)
            }
            #[cfg(feature = "roland")]
            Self::MoveRobotByAngle(c) => {
                write!(w, "{}{}", cmd::MoveRobotByAngle::PREFIX, SEPARATOR)?;
                c.write_text(w)
            }
            #[cfg(feature = "roland")]
            Self::StopRobot(c) => {
                write!(w, "{}{}", cmd::StopRobot::PREFIX, SEPARATOR)?;
                c.write_text(w)
            }
            #[cfg(feature = "roland")]
            Self::Led(c) => {
                write!(w, "{}{}", cmd::Led::PREFIX, SEPARATOR)?;
                c.write_text(w)
            }
            #[cfg(feature = "roland")]
            Self::ServoAbsolute(c) => {
                write!(w, "{}{}", cmd::ServoAbsolute::PREFIX, SEPARATOR)?;
                c.write_text(w)
            }
            #[cfg(feature = "roland")]
            Self::Buzzer(c) => {
                write!(w, "{}{}", cmd::Buzzer::PREFIX, SEPARATOR)?;
                c.write_text(w)
            }
            #[cfg(feature = "roland")]
            Self::TrackSensor(c) => {
                write!(w, "{}{}", cmd::TrackSensor::PREFIX, SEPARATOR)?;
                c.write_text(w)
            }
            #[cfg(feature = "roland")]
            Self::UltraSensor(c) => {
                write!(w, "{}{}", cmd::UltraSensor::PREFIX, SEPARATOR)?;
                c.write_text(w)
            }

            #[cfg(feature = "gpio")]
            Self::ReadPin(c) => {
                write!(w, "{}{}", cmd::ReadPin::PREFIX, SEPARATOR)?;
                c.write_text(w)
            }
            #[cfg(feature = "gpio")]
            Self::SetPin(c) => {
                write!(w, "{}{}", cmd::SetPin::PREFIX, SEPARATOR)?;
                c.write_text(w)
            }
            #[cfg(feature = "gpio")]
            Self::SetPwm(c) => {
                write!(w, "{}{}", cmd::SetPwm::PREFIX, SEPARATOR)?;
                c.write_text(w)
            }
            #[cfg(feature = "gpio")]
            Self::ServoBasic(c) => {
                write!(w, "{}{}", cmd::ServoBasic::PREFIX, SEPARATOR)?;
                c.write_text(w)
            }

            #[cfg(feature = "camloc")]
            Self::GetPosition(c) => {
                write!(w, "{}{}", cmd::GetPosition::PREFIX, SEPARATOR)?;
                c.write_text(w)
            }

            Self::Subscribe(c) => {
                write!(w, "{}{}", cmd::Subscribe::PREFIX, SEPARATOR)?;
                c.write_text(w)
            }
            Self::Unsubscribe(c) => {
                write!(w, "{}{}", cmd::Unsubscribe::PREFIX, SEPARATOR)?;
                c.write_text(w)
            }
            Self::Nop(c) => {
                write!(w, "{}{}", cmd::Nop::PREFIX, SEPARATOR)?;
                c.write_text(w)
            }
            Self::GetUptime(c) => {
                write!(w, "{}{}", cmd::GetUptime::PREFIX, SEPARATOR)?;
                c.write_text(w)
            }
        }
    }
    fn write_binary(&self, w: &mut dyn std::io::Write) -> anyhow::Result<()> {
        match self {
            #[cfg(feature = "roland")]
            Self::MoveRobot(c) => {
                w.write_all(&(cmd::MoveRobot::PREFIX as u8).to_be_bytes())?;
                c.write_binary(w)
            }
            #[cfg(feature = "roland")]
            Self::MoveRobotByAngle(c) => {
                w.write_all(&(cmd::MoveRobotByAngle::PREFIX as u8).to_be_bytes())?;
                c.write_binary(w)
            }
            #[cfg(feature = "roland")]
            Self::StopRobot(c) => {
                w.write_all(&(cmd::StopRobot::PREFIX as u8).to_be_bytes())?;
                c.write_binary(w)
            }
            #[cfg(feature = "roland")]
            Self::Led(c) => {
                w.write_all(&(cmd::Led::PREFIX as u8).to_be_bytes())?;
                c.write_binary(w)
            }
            #[cfg(feature = "roland")]
            Self::ServoAbsolute(c) => {
                w.write_all(&(cmd::ServoAbsolute::PREFIX as u8).to_be_bytes())?;
                c.write_binary(w)
            }
            #[cfg(feature = "roland")]
            Self::Buzzer(c) => {
                w.write_all(&(cmd::Buzzer::PREFIX as u8).to_be_bytes())?;
                c.write_binary(w)
            }
            #[cfg(feature = "roland")]
            Self::TrackSensor(c) => {
                w.write_all(&(cmd::TrackSensor::PREFIX as u8).to_be_bytes())?;
                c.write_binary(w)
            }
            #[cfg(feature = "roland")]
            Self::UltraSensor(c) => {
                w.write_all(&(cmd::UltraSensor::PREFIX as u8).to_be_bytes())?;
                c.write_binary(w)
            }

            #[cfg(feature = "gpio")]
            Self::ReadPin(c) => {
                w.write_all(&(cmd::ReadPin::PREFIX as u8).to_be_bytes())?;
                c.write_binary(w)
            }
            #[cfg(feature = "gpio")]
            Self::SetPin(c) => {
                w.write_all(&(cmd::SetPin::PREFIX as u8).to_be_bytes())?;
                c.write_binary(w)
            }
            #[cfg(feature = "gpio")]
            Self::SetPwm(c) => {
                w.write_all(&(cmd::SetPwm::PREFIX as u8).to_be_bytes())?;
                c.write_binary(w)
            }
            #[cfg(feature = "gpio")]
            Self::ServoBasic(c) => {
                w.write_all(&(cmd::ServoBasic::PREFIX as u8).to_be_bytes())?;
                c.write_binary(w)
            }

            #[cfg(feature = "camloc")]
            Self::GetPosition(c) => {
                w.write_all(&(cmd::GetPosition::PREFIX as u8).to_be_bytes())?;
                c.write_binary(w)
            }

            Self::Subscribe(c) => {
                w.write_all(&(cmd::Subscribe::PREFIX as u8).to_be_bytes())?;
                c.write_binary(w)
            }
            Self::Unsubscribe(c) => {
                w.write_all(&(cmd::Unsubscribe::PREFIX as u8).to_be_bytes())?;
                c.write_binary(w)
            }
            Self::Nop(c) => {
                w.write_all(&(cmd::Nop::PREFIX as u8).to_be_bytes())?;
                c.write_binary(w)
            }
            Self::GetUptime(c) => {
                w.write_all(&(cmd::GetUptime::PREFIX as u8).to_be_bytes())?;
                c.write_binary(w)
            }
        }
    }
    #[cfg(feature = "async")]
    async fn write_binary_async(
        &self,
        w: &mut (dyn ::futures::AsyncWrite + Send + Unpin),
    ) -> ::anyhow::Result<()> {
        use futures::AsyncWriteExt;
        match self {
            #[cfg(feature = "roland")]
            Self::MoveRobot(c) => {
                w.write_all(&(cmd::MoveRobot::PREFIX as u8).to_be_bytes())
                    .await?;
                c.write_binary_async(w).await
            }
            #[cfg(feature = "roland")]
            Self::MoveRobotByAngle(c) => {
                w.write_all(&(cmd::MoveRobotByAngle::PREFIX as u8).to_be_bytes())
                    .await?;
                c.write_binary_async(w).await
            }
            #[cfg(feature = "roland")]
            Self::StopRobot(c) => {
                w.write_all(&(cmd::StopRobot::PREFIX as u8).to_be_bytes())
                    .await?;
                c.write_binary_async(w).await
            }
            #[cfg(feature = "roland")]
            Self::Led(c) => {
                w.write_all(&(cmd::Led::PREFIX as u8).to_be_bytes()).await?;
                c.write_binary_async(w).await
            }
            #[cfg(feature = "roland")]
            Self::ServoAbsolute(c) => {
                w.write_all(&(cmd::ServoAbsolute::PREFIX as u8).to_be_bytes())
                    .await?;
                c.write_binary_async(w).await
            }
            #[cfg(feature = "roland")]
            Self::Buzzer(c) => {
                w.write_all(&(cmd::Buzzer::PREFIX as u8).to_be_bytes())
                    .await?;
                c.write_binary_async(w).await
            }
            #[cfg(feature = "roland")]
            Self::TrackSensor(c) => {
                w.write_all(&(cmd::TrackSensor::PREFIX as u8).to_be_bytes())
                    .await?;
                c.write_binary_async(w).await
            }
            #[cfg(feature = "roland")]
            Self::UltraSensor(c) => {
                w.write_all(&(cmd::UltraSensor::PREFIX as u8).to_be_bytes())
                    .await?;
                c.write_binary_async(w).await
            }

            #[cfg(feature = "gpio")]
            Self::ReadPin(c) => {
                w.write_all(&(cmd::ReadPin::PREFIX as u8).to_be_bytes())
                    .await?;
                c.write_binary_async(w).await
            }
            #[cfg(feature = "gpio")]
            Self::SetPin(c) => {
                w.write_all(&(cmd::SetPin::PREFIX as u8).to_be_bytes())
                    .await?;
                c.write_binary_async(w).await
            }
            #[cfg(feature = "gpio")]
            Self::SetPwm(c) => {
                w.write_all(&(cmd::SetPwm::PREFIX as u8).to_be_bytes())
                    .await?;
                c.write_binary_async(w).await
            }
            #[cfg(feature = "gpio")]
            Self::ServoBasic(c) => {
                w.write_all(&(cmd::ServoBasic::PREFIX as u8).to_be_bytes())
                    .await?;
                c.write_binary_async(w).await
            }

            #[cfg(feature = "camloc")]
            Self::GetPosition(c) => {
                w.write_all(&(cmd::GetPosition::PREFIX as u8).to_be_bytes())
                    .await?;
                c.write_binary_async(w).await
            }

            Self::Subscribe(c) => {
                w.write_all(&(cmd::Subscribe::PREFIX as u8).to_be_bytes())
                    .await?;
                c.write_binary_async(w).await
            }
            Self::Unsubscribe(c) => {
                w.write_all(&(cmd::Unsubscribe::PREFIX as u8).to_be_bytes())
                    .await?;
                c.write_binary_async(w).await
            }
            Self::Nop(c) => {
                w.write_all(&(cmd::Nop::PREFIX as u8).to_be_bytes()).await?;
                c.write_binary_async(w).await
            }
            Self::GetUptime(c) => {
                w.write_all(&(cmd::GetUptime::PREFIX as u8).to_be_bytes())
                    .await?;
                c.write_binary_async(w).await
            }
        }
    }
}

impl Concrete {
    pub fn get_prefix(&self) -> char {
        match self {
            #[cfg(feature = "roland")]
            Self::MoveRobot(_) => cmd::MoveRobot::PREFIX,
            #[cfg(feature = "roland")]
            Self::MoveRobotByAngle(_) => cmd::MoveRobotByAngle::PREFIX,
            #[cfg(feature = "roland")]
            Self::StopRobot(_) => cmd::StopRobot::PREFIX,
            #[cfg(feature = "roland")]
            Self::Led(_) => cmd::Led::PREFIX,
            #[cfg(feature = "roland")]
            Self::ServoAbsolute(_) => cmd::ServoAbsolute::PREFIX,
            #[cfg(feature = "roland")]
            Self::Buzzer(_) => cmd::Buzzer::PREFIX,
            #[cfg(feature = "roland")]
            Self::TrackSensor(_) => cmd::TrackSensor::PREFIX,
            #[cfg(feature = "roland")]
            Self::UltraSensor(_) => cmd::UltraSensor::PREFIX,

            #[cfg(feature = "gpio")]
            Self::ReadPin(_) => cmd::ReadPin::PREFIX,
            #[cfg(feature = "gpio")]
            Self::SetPin(_) => cmd::SetPin::PREFIX,
            #[cfg(feature = "gpio")]
            Self::SetPwm(_) => cmd::SetPwm::PREFIX,
            #[cfg(feature = "gpio")]
            Self::ServoBasic(_) => cmd::ServoBasic::PREFIX,

            #[cfg(feature = "camloc")]
            Self::GetPosition(_) => cmd::GetPosition::PREFIX,

            Self::Subscribe(_) => cmd::Subscribe::PREFIX,
            Self::Unsubscribe(_) => cmd::Unsubscribe::PREFIX,
            Self::Nop(_) => cmd::Nop::PREFIX,
            Self::GetUptime(_) => cmd::GetUptime::PREFIX,
        }
    }

    pub fn has_return(&self) -> bool {
        use super::has_return as has;
        match self {
            #[cfg(feature = "roland")]
            Self::MoveRobot(_) => has::<cmd::MoveRobot>(),
            #[cfg(feature = "roland")]
            Self::MoveRobotByAngle(_) => has::<cmd::MoveRobotByAngle>(),
            #[cfg(feature = "roland")]
            Self::StopRobot(_) => has::<cmd::StopRobot>(),
            #[cfg(feature = "roland")]
            Self::Led(_) => has::<cmd::Led>(),
            #[cfg(feature = "roland")]
            Self::ServoAbsolute(_) => has::<cmd::ServoAbsolute>(),
            #[cfg(feature = "roland")]
            Self::Buzzer(_) => has::<cmd::Buzzer>(),
            #[cfg(feature = "roland")]
            Self::TrackSensor(_) => has::<cmd::TrackSensor>(),
            #[cfg(feature = "roland")]
            Self::UltraSensor(_) => has::<cmd::UltraSensor>(),

            #[cfg(feature = "gpio")]
            Self::ReadPin(_) => has::<cmd::ReadPin>(),
            #[cfg(feature = "gpio")]
            Self::SetPin(_) => has::<cmd::SetPin>(),
            #[cfg(feature = "gpio")]
            Self::SetPwm(_) => has::<cmd::SetPwm>(),
            #[cfg(feature = "gpio")]
            Self::ServoBasic(_) => has::<cmd::ServoBasic>(),

            #[cfg(feature = "camloc")]
            Self::GetPosition(_) => has::<cmd::GetPosition>(),

            Self::Subscribe(_) => has::<cmd::Subscribe>(),
            Self::Unsubscribe(_) => has::<cmd::Unsubscribe>(),
            Self::Nop(_) => has::<cmd::Nop>(),
            Self::GetUptime(_) => has::<cmd::GetUptime>(),
        }
    }
}

impl Display for Concrete {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.write_text(f)
    }
}
