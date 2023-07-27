use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::cmd::{self, Command};

#[derive(Serialize, Deserialize)]
pub struct Concr {
    pub prefix: char,
    pub cmd: Concrete,
}

impl Concr {
    pub fn new(cmd: Concrete) -> Self {
        Self {
            prefix: cmd.get_prefix(),
            cmd,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
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

impl Display for Concr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        crate::text_format::ser::write(self, f)
    }
}
