pub mod de;
pub mod error;
pub mod ser;

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::{
        cmd::{self, Concrete},
        event::{self, ConcreteType},
    };
    use rand::random;

    #[test]
    fn ser_matches_de_event() -> anyhow::Result<()> {
        for _ in 0..100 {
            let cs = [
                ConcreteType::TrackSensor(event::TrackSensor),
                ConcreteType::UltraSensor(event::UltraSensor(Duration::from_secs_f64(random()))),
                ConcreteType::GpioPin(event::GpioPin(random())),
                ConcreteType::CamlocConnect(event::CamlocConnect),
                ConcreteType::CamlocDisconnect(event::CamlocDisconnect),
                ConcreteType::CamlocPosition(event::CamlocPosition),
                ConcreteType::CamlocInfoUpdate(event::CamlocInfoUpdate),
            ];

            for c in cs {
                let txt1 = super::ser::to_string(&c)?;
                let concrete = super::de::from_str::<ConcreteType>(&txt1)?;
                let txt2 = super::ser::to_string(&concrete)?;
                if txt1 != txt2 {
                    println!("❌ {txt1} | {txt2}");
                }
            }
        }

        Ok(())
    }

    #[test]
    fn ser_matches_de_cmd() -> anyhow::Result<()> {
        for _ in 0..100 {
            let cs = [
                Concrete::MoveRobot(cmd::MoveRobot(random(), random())),
                Concrete::MoveRobotByAngle(cmd::MoveRobotByAngle(random(), random())),
                Concrete::StopRobot(cmd::StopRobot),
                Concrete::Led(cmd::Led(random(), random(), random())),
                Concrete::RolandServo(cmd::RolandServo(random())),
                Concrete::Buzzer(cmd::Buzzer(random())),
                Concrete::TrackSensor(cmd::TrackSensor),
                Concrete::UltraSensor(cmd::UltraSensor),
                Concrete::PinMode(cmd::PinMode(
                    random(),
                    if random::<bool>() {
                        crate::gpio::Mode::Input
                    } else {
                        crate::gpio::Mode::Output
                    },
                )),
                Concrete::ReadPin(cmd::ReadPin(random())),
                Concrete::WritePin(cmd::WritePin(random(), random())),
                Concrete::Pwm(cmd::Pwm(random(), random(), random())),
                Concrete::Servo(cmd::Servo(random(), random())),
                Concrete::GetPosition(cmd::GetPosition),
                Concrete::Subscribe(cmd::Subscribe(event::GpioPin(random()).into())),
                Concrete::Unsubscribe(cmd::Unsubscribe(event::GpioPin(random()).into())),
                Concrete::Nop(cmd::Nop),
                Concrete::GetUptime(cmd::GetUptime),
                Concrete::Abort(cmd::Abort),
            ];

            for c in cs {
                let txt1 = super::ser::to_string(&c)?;
                let concrete = super::de::from_str::<Concrete>(&txt1)?;
                let txt2 = super::ser::to_string(&concrete)?;
                if txt1 != txt2 {
                    println!("❌ {txt1} | {txt2}");
                }
            }
        }

        Ok(())
    }
}
