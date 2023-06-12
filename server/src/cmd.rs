use std::time::Instant;

use crate::Robot;
use roblib::cmd::Cmd;

#[cfg(all(feature = "gpio", feature = "backend"))]
use roblib::gpio::Gpio;

#[cfg(all(feature = "roland", feature = "backend"))]
use roblib::roland::Roland;

#[allow(unused_variables)]
pub(crate) async fn execute_command(cmd: &Cmd, robot: &Robot) -> anyhow::Result<Option<String>> {
    let res = match *cmd {
        #[cfg(feature = "roland")]
        Cmd::MoveRobot(left, right) => {
            debug!("Moving robot: {left}:{right}");

            #[cfg(feature = "backend")]
            if let Some(r) = &robot.roland {
                #[allow(clippy::let_unit_value)]
                let hint = r.drive(left, right)?;

                #[cfg(feature = "camloc")]
                if let Some(c) = &robot.camloc {
                    c.service.set_motion_hint(hint).await;
                }
            }

            None
        }

        #[cfg(feature = "roland")]
        Cmd::MoveRobotByAngle(angle, speed) => {
            debug!("Moving robot by angle: {}:{}", angle, speed);

            #[cfg(feature = "backend")]
            if let Some(r) = &robot.roland {
                #[allow(clippy::let_unit_value)]
                let hint = r.drive_by_angle(angle, speed)?;

                #[cfg(feature = "camloc")]
                if let Some(c) = &robot.camloc {
                    c.service.set_motion_hint(hint).await;
                }
            }
            None
        }

        #[cfg(feature = "roland")]
        Cmd::StopRobot => {
            debug!("Stopping robot");

            #[cfg(feature = "backend")]
            if let Some(r) = &robot.roland {
                r.drive(0., 0.)?;
            }

            None
        }

        #[cfg(feature = "roland")]
        Cmd::Led(r, g, b) => {
            debug!("LED: {r}:{g}:{b}");

            #[cfg(feature = "backend")]
            if let Some(rr) = &robot.roland {
                rr.led(r, g, b)?;
            }

            None
        }

        #[cfg(feature = "roland")]
        Cmd::ServoAbsolute(deg) => {
            debug!("Servo absolute: {deg}");

            #[cfg(feature = "backend")]
            if let Some(r) = &robot.roland {
                r.servo(deg)?;
            }

            None
        }

        #[cfg(feature = "roland")]
        Cmd::Buzzer(pw) => {
            debug!("Buzzer: {pw}");

            #[cfg(feature = "backend")]
            if let Some(r) = &robot.roland {
                r.buzzer(pw)?
            }

            None
        }

        #[cfg(feature = "roland")]
        Cmd::TrackSensor => {
            debug!("Track sensor");

            #[cfg(feature = "backend")]
            let res = if let Some(r) = &robot.roland {
                r.track_sensor()?
            } else {
                [false, false, false, false]
            }
            .map(|b| b as u8);

            #[cfg(not(feature = "backend"))]
            let res = [false, false, false];

            Some(format!("{},{},{},{}", res[0], res[1], res[2], res[3]))
        }

        #[cfg(feature = "roland")]
        Cmd::UltraSensor => {
            debug!("Ultra sensor");

            #[cfg(feature = "backend")]
            let res = if let Some(r) = &robot.roland {
                r.ultra_sensor()?
            } else {
                f64::NAN
            };

            #[cfg(not(feature = "backend"))]
            let res = f64::NAN;

            Some(format!("{res}"))
        }

        #[cfg(feature = "camloc")]
        Cmd::GetPosition => {
            debug!("Get position");

            let res = if let Some(c) = &robot.camloc {
                c.service.get_position().await.map(|tp| tp.position)
            } else {
                None
            };

            Some(todo!())
        }

        #[cfg(feature = "gpio")]
        Cmd::ReadPin(pin) => {
            debug!("Read pin: {pin}");

            #[cfg(feature = "backend")]
            let res = if let Some(r) = &robot.raw_gpio {
                Some(format!("{}", r.read_pin(pin)? as u8))
            } else {
                Some("0".to_string())
            };

            #[cfg(not(feature = "backend"))]
            let res = Some("0".to_string());

            res
        }

        #[cfg(feature = "gpio")]
        Cmd::SetPin(pin, value) => {
            debug!("Set pin: {pin}:{value}");

            #[cfg(feature = "backend")]
            if let Some(r) = &robot.raw_gpio {
                r.set_pin(pin, value)?;
            }

            None
        }

        #[cfg(feature = "gpio")]
        Cmd::SetPwm(pin, hz, cycle) => {
            debug!("Set pwm: {pin}:{hz}:{cycle}");

            #[cfg(feature = "backend")]
            if let Some(r) = &robot.raw_gpio {
                r.pwm(pin, hz, cycle)?;
            }

            None
        }

        #[cfg(feature = "gpio")]
        Cmd::ServoBasic(pin, deg) => {
            debug!("Servo basic: {deg}");

            #[cfg(feature = "backend")]
            if let Some(r) = &robot.raw_gpio {
                r.servo(pin, deg)?;
            }

            None
        }

        Cmd::Nop => {
            debug!("Nop");
            None
        }

        Cmd::GetUptime => {
            debug!("Get uptime");

            Some(format!(
                "{}",
                (Instant::now() - robot.startup_time).as_millis()
            ))
        }
    };
    Ok(res)
}
