use std::fmt::Display;

use serde::{
    de::{self, SeqAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Serialize,
};

use crate::cmd::{self, Command};

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
    RolandServo(cmd::RolandServo),
    #[cfg(feature = "roland")]
    Buzzer(cmd::Buzzer),
    #[cfg(feature = "roland")]
    TrackSensor(cmd::TrackSensor),
    #[cfg(feature = "roland")]
    UltraSensor(cmd::UltraSensor),

    #[cfg(feature = "gpio")]
    PinMode(cmd::PinMode),
    #[cfg(feature = "gpio")]
    ReadPin(cmd::ReadPin),
    #[cfg(feature = "gpio")]
    WritePin(cmd::WritePin),
    #[cfg(feature = "gpio")]
    Pwm(cmd::Pwm),
    #[cfg(feature = "gpio")]
    Servo(cmd::Servo),

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
            Self::RolandServo(_) => cmd::RolandServo::PREFIX,
            #[cfg(feature = "roland")]
            Self::Buzzer(_) => cmd::Buzzer::PREFIX,
            #[cfg(feature = "roland")]
            Self::TrackSensor(_) => cmd::TrackSensor::PREFIX,
            #[cfg(feature = "roland")]
            Self::UltraSensor(_) => cmd::UltraSensor::PREFIX,

            #[cfg(feature = "gpio")]
            Self::PinMode(_) => cmd::PinMode::PREFIX,
            #[cfg(feature = "gpio")]
            Self::ReadPin(_) => cmd::ReadPin::PREFIX,
            #[cfg(feature = "gpio")]
            Self::WritePin(_) => cmd::WritePin::PREFIX,
            #[cfg(feature = "gpio")]
            Self::Pwm(_) => cmd::Pwm::PREFIX,
            #[cfg(feature = "gpio")]
            Self::Servo(_) => cmd::Servo::PREFIX,

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
            Self::RolandServo(_) => has::<cmd::RolandServo>(),
            #[cfg(feature = "roland")]
            Self::Buzzer(_) => has::<cmd::Buzzer>(),
            #[cfg(feature = "roland")]
            Self::TrackSensor(_) => has::<cmd::TrackSensor>(),
            #[cfg(feature = "roland")]
            Self::UltraSensor(_) => has::<cmd::UltraSensor>(),

            #[cfg(feature = "gpio")]
            Self::PinMode(_) => has::<cmd::PinMode>(),
            #[cfg(feature = "gpio")]
            Self::ReadPin(_) => has::<cmd::ReadPin>(),
            #[cfg(feature = "gpio")]
            Self::WritePin(_) => has::<cmd::WritePin>(),
            #[cfg(feature = "gpio")]
            Self::Pwm(_) => has::<cmd::Pwm>(),
            #[cfg(feature = "gpio")]
            Self::Servo(_) => has::<cmd::Servo>(),

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
        let mut s = serializer.serialize_struct("Concrete", 2)?;
        match self {
            #[cfg(feature = "roland")]
            Self::MoveRobot(c) => {
                s.serialize_field("prefix", &cmd::MoveRobot::PREFIX)?;
                s.serialize_field("cmd", &c)?;
            }
            #[cfg(feature = "roland")]
            Self::MoveRobotByAngle(c) => {
                s.serialize_field("prefix", &cmd::MoveRobotByAngle::PREFIX)?;
                s.serialize_field("cmd", &c)?;
            }
            #[cfg(feature = "roland")]
            Self::StopRobot(c) => {
                s.serialize_field("prefix", &cmd::StopRobot::PREFIX)?;
                s.serialize_field("cmd", &c)?;
            }
            #[cfg(feature = "roland")]
            Self::Led(c) => {
                s.serialize_field("prefix", &cmd::Led::PREFIX)?;
                s.serialize_field("cmd", &c)?;
            }
            #[cfg(feature = "roland")]
            Self::RolandServo(c) => {
                s.serialize_field("prefix", &cmd::RolandServo::PREFIX)?;
                s.serialize_field("cmd", &c)?;
            }
            #[cfg(feature = "roland")]
            Self::Buzzer(c) => {
                s.serialize_field("prefix", &cmd::Buzzer::PREFIX)?;
                s.serialize_field("cmd", &c)?;
            }
            #[cfg(feature = "roland")]
            Self::TrackSensor(c) => {
                s.serialize_field("prefix", &cmd::TrackSensor::PREFIX)?;
                s.serialize_field("cmd", &c)?;
            }
            #[cfg(feature = "roland")]
            Self::UltraSensor(c) => {
                s.serialize_field("prefix", &cmd::UltraSensor::PREFIX)?;
                s.serialize_field("cmd", &c)?;
            }

            #[cfg(feature = "gpio")]
            Self::PinMode(c) => {
                s.serialize_field("prefix", &cmd::PinMode::PREFIX)?;
                s.serialize_field("cmd", &c)?;
            }
            #[cfg(feature = "gpio")]
            Self::ReadPin(c) => {
                s.serialize_field("prefix", &cmd::ReadPin::PREFIX)?;
                s.serialize_field("cmd", &c)?;
            }
            #[cfg(feature = "gpio")]
            Self::WritePin(c) => {
                s.serialize_field("prefix", &cmd::WritePin::PREFIX)?;
                s.serialize_field("cmd", &c)?;
            }
            #[cfg(feature = "gpio")]
            Self::Pwm(c) => {
                s.serialize_field("prefix", &cmd::Pwm::PREFIX)?;
                s.serialize_field("cmd", &c)?;
            }
            #[cfg(feature = "gpio")]
            Self::Servo(c) => {
                s.serialize_field("prefix", &cmd::Servo::PREFIX)?;
                s.serialize_field("cmd", &c)?;
            }

            #[cfg(feature = "camloc")]
            Self::GetPosition(c) => {
                s.serialize_field("prefix", &cmd::GetPosition::PREFIX)?;
                s.serialize_field("cmd", &c)?;
            }

            Self::Subscribe(c) => {
                s.serialize_field("prefix", &cmd::Subscribe::PREFIX)?;
                s.serialize_field("cmd", &c)?;
            }
            Self::Unsubscribe(c) => {
                s.serialize_field("prefix", &cmd::Unsubscribe::PREFIX)?;
                s.serialize_field("cmd", &c)?;
            }
            Self::Nop(c) => {
                s.serialize_field("prefix", &cmd::Nop::PREFIX)?;
                s.serialize_field("cmd", &c)?;
            }
            Self::GetUptime(c) => {
                s.serialize_field("prefix", &cmd::GetUptime::PREFIX)?;
                s.serialize_field("cmd", &c)?;
            }
        }
        s.end()
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
                    cmd::RolandServo::PREFIX => {
                        let cmd = seq
                            .next_element()?
                            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                        Ok(Concrete::RolandServo(cmd))
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
                    cmd::PinMode::PREFIX => {
                        let cmd = seq
                            .next_element()?
                            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                        Ok(Concrete::PinMode(cmd))
                    }
                    #[cfg(feature = "gpio")]
                    cmd::ReadPin::PREFIX => {
                        let cmd = seq
                            .next_element()?
                            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                        Ok(Concrete::ReadPin(cmd))
                    }
                    #[cfg(feature = "gpio")]
                    cmd::WritePin::PREFIX => {
                        let cmd = seq
                            .next_element()?
                            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                        Ok(Concrete::WritePin(cmd))
                    }
                    #[cfg(feature = "gpio")]
                    cmd::Pwm::PREFIX => {
                        let cmd = seq
                            .next_element()?
                            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                        Ok(Concrete::Pwm(cmd))
                    }
                    #[cfg(feature = "gpio")]
                    cmd::Servo::PREFIX => {
                        let cmd = seq
                            .next_element()?
                            .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                        Ok(Concrete::Servo(cmd))
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
