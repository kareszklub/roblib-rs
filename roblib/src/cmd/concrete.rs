use std::fmt::Display;

use serde::{
    de::{self, SeqAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Serialize,
};

use crate::cmd::{self, Command};

#[derive(Serialize, Deserialize)]
pub struct Concr {
    pub id: u32,
    pub cmd: Concrete,
}

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

impl Serialize for Concrete {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            #[cfg(feature = "roland")]
            Self::MoveRobot(c) => {
                let mut s = serializer.serialize_struct("Concrete", 2)?;
                s.serialize_field("prefix", &cmd::MoveRobot::PREFIX)?;
                s.serialize_field("cmd", &c)?;
                s.end()
            }
            #[cfg(feature = "roland")]
            Self::MoveRobotByAngle(c) => {
                let mut s = serializer.serialize_struct("Concrete", 2)?;
                s.serialize_field("prefix", &cmd::MoveRobotByAngle::PREFIX)?;
                s.serialize_field("cmd", &c)?;
                s.end()
            }
            #[cfg(feature = "roland")]
            Self::StopRobot(c) => {
                let mut s = serializer.serialize_struct("Concrete", 2)?;
                s.serialize_field("prefix", &cmd::StopRobot::PREFIX)?;
                s.serialize_field("cmd", &c)?;
                s.end()
            }
            #[cfg(feature = "roland")]
            Self::Led(c) => {
                let mut s = serializer.serialize_struct("Concrete", 2)?;
                s.serialize_field("prefix", &cmd::Led::PREFIX)?;
                s.serialize_field("cmd", &c)?;
                s.end()
            }
            #[cfg(feature = "roland")]
            Self::ServoAbsolute(c) => {
                let mut s = serializer.serialize_struct("Concrete", 2)?;
                s.serialize_field("prefix", &cmd::ServoAbsolute::PREFIX)?;
                s.serialize_field("cmd", &c)?;
                s.end()
            }
            #[cfg(feature = "roland")]
            Self::Buzzer(c) => {
                let mut s = serializer.serialize_struct("Concrete", 2)?;
                s.serialize_field("prefix", &cmd::Buzzer::PREFIX)?;
                s.serialize_field("cmd", &c)?;
                s.end()
            }
            #[cfg(feature = "roland")]
            Self::TrackSensor(c) => {
                let mut s = serializer.serialize_struct("Concrete", 2)?;
                s.serialize_field("prefix", &cmd::TrackSensor::PREFIX)?;
                s.serialize_field("cmd", &c)?;
                s.end()
            }
            #[cfg(feature = "roland")]
            Self::UltraSensor(c) => {
                let mut s = serializer.serialize_struct("Concrete", 2)?;
                s.serialize_field("prefix", &cmd::UltraSensor::PREFIX)?;
                s.serialize_field("cmd", &c)?;
                s.end()
            }

            #[cfg(feature = "gpio")]
            Self::ReadPin(c) => {
                let mut s = serializer.serialize_struct("Concrete", 2)?;
                s.serialize_field("prefix", &cmd::ReadPin::PREFIX)?;
                s.serialize_field("cmd", &c)?;
                s.end()
            }
            #[cfg(feature = "gpio")]
            Self::SetPin(c) => {
                let mut s = serializer.serialize_struct("Concrete", 2)?;
                s.serialize_field("prefix", &cmd::SetPin::PREFIX)?;
                s.serialize_field("cmd", &c)?;
                s.end()
            }
            #[cfg(feature = "gpio")]
            Self::SetPwm(c) => {
                let mut s = serializer.serialize_struct("Concrete", 2)?;
                s.serialize_field("prefix", &cmd::SetPwm::PREFIX)?;
                s.serialize_field("cmd", &c)?;
                s.end()
            }
            #[cfg(feature = "gpio")]
            Self::ServoBasic(c) => {
                let mut s = serializer.serialize_struct("Concrete", 2)?;
                s.serialize_field("prefix", &cmd::ServoBasic::PREFIX)?;
                s.serialize_field("cmd", &c)?;
                s.end()
            }

            #[cfg(feature = "camloc")]
            Self::GetPosition(c) => {
                let mut s = serializer.serialize_struct("Concrete", 2)?;
                s.serialize_field("prefix", &cmd::GetPosition::PREFIX)?;
                s.serialize_field("cmd", &c)?;
                s.end()
            }

            Self::Subscribe(c) => {
                let mut s = serializer.serialize_struct("Concrete", 2)?;
                s.serialize_field("prefix", &cmd::Subscribe::PREFIX)?;
                s.serialize_field("cmd", &c)?;
                s.end()
            }
            Self::Unsubscribe(c) => {
                let mut s = serializer.serialize_struct("Concrete", 2)?;
                s.serialize_field("prefix", &cmd::Unsubscribe::PREFIX)?;
                s.serialize_field("cmd", &c)?;
                s.end()
            }
            Self::Nop(c) => {
                let mut s = serializer.serialize_struct("Concrete", 2)?;
                s.serialize_field("prefix", &cmd::Nop::PREFIX)?;
                s.serialize_field("cmd", &c)?;
                s.end()
            }
            Self::GetUptime(c) => {
                let mut s = serializer.serialize_struct("Concrete", 2)?;
                s.serialize_field("prefix", &cmd::GetUptime::PREFIX)?;
                s.serialize_field("cmd", &c)?;
                s.end()
            }
        }
    }
}
impl<'de> Deserialize<'de> for Concrete {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ConcreteVisitor;
        impl<'de> Visitor<'de> for ConcreteVisitor {
            type Value = Concrete;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a prefix and a command body")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let prefix: char = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;

                match prefix {
                    #[cfg(feature = "roland")]
                    cmd::MoveRobot::PREFIX => {
                        let cmd = seq
                            .next_element()?
                            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                        Ok(Concrete::MoveRobot(cmd))
                    }
                    #[cfg(feature = "roland")]
                    cmd::MoveRobotByAngle::PREFIX => {
                        let cmd = seq
                            .next_element()?
                            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                        Ok(Concrete::MoveRobotByAngle(cmd))
                    }
                    #[cfg(feature = "roland")]
                    cmd::StopRobot::PREFIX => {
                        let cmd = seq
                            .next_element()?
                            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                        Ok(Concrete::StopRobot(cmd))
                    }
                    #[cfg(feature = "roland")]
                    cmd::Led::PREFIX => {
                        let cmd = seq
                            .next_element()?
                            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                        Ok(Concrete::Led(cmd))
                    }
                    #[cfg(feature = "roland")]
                    cmd::ServoAbsolute::PREFIX => {
                        let cmd = seq
                            .next_element()?
                            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                        Ok(Concrete::ServoAbsolute(cmd))
                    }
                    #[cfg(feature = "roland")]
                    cmd::Buzzer::PREFIX => {
                        let cmd = seq
                            .next_element()?
                            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                        Ok(Concrete::Buzzer(cmd))
                    }
                    #[cfg(feature = "roland")]
                    cmd::TrackSensor::PREFIX => {
                        let cmd = seq
                            .next_element()?
                            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                        Ok(Concrete::TrackSensor(cmd))
                    }
                    #[cfg(feature = "roland")]
                    cmd::UltraSensor::PREFIX => {
                        let cmd = seq
                            .next_element()?
                            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                        Ok(Concrete::UltraSensor(cmd))
                    }

                    #[cfg(feature = "gpio")]
                    cmd::ReadPin::PREFIX => {
                        let cmd = seq
                            .next_element()?
                            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                        Ok(Concrete::ReadPin(cmd))
                    }
                    #[cfg(feature = "gpio")]
                    cmd::SetPin::PREFIX => {
                        let cmd = seq
                            .next_element()?
                            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                        Ok(Concrete::SetPin(cmd))
                    }
                    #[cfg(feature = "gpio")]
                    cmd::SetPwm::PREFIX => {
                        let cmd = seq
                            .next_element()?
                            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                        Ok(Concrete::SetPwm(cmd))
                    }
                    #[cfg(feature = "gpio")]
                    cmd::ServoBasic::PREFIX => {
                        let cmd = seq
                            .next_element()?
                            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                        Ok(Concrete::ServoBasic(cmd))
                    }

                    #[cfg(feature = "camloc")]
                    cmd::GetPosition::PREFIX => {
                        let cmd = seq
                            .next_element()?
                            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                        Ok(Concrete::GetPosition(cmd))
                    }

                    cmd::Subscribe::PREFIX => {
                        let cmd = seq
                            .next_element()?
                            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                        Ok(Concrete::Subscribe(cmd))
                    }
                    cmd::Unsubscribe::PREFIX => {
                        let cmd = seq
                            .next_element()?
                            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                        Ok(Concrete::Unsubscribe(cmd))
                    }
                    cmd::Nop::PREFIX => {
                        let cmd = seq
                            .next_element()?
                            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                        Ok(Concrete::Nop(cmd))
                    }
                    cmd::GetUptime::PREFIX => {
                        let cmd = seq
                            .next_element()?
                            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                        Ok(Concrete::GetUptime(cmd))
                    }

                    _ => Err(de::Error::invalid_value(
                        de::Unexpected::Char(prefix),
                        &"a command prefix",
                    )),
                }
            }
        }

        deserializer.deserialize_struct("Concrete", &["prefix", "cmd"], ConcreteVisitor)
    }
}

impl Display for Concrete {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
