use roblib::cmd::{parsing::commands::Concrete, SEPARATOR};
use roblib_client::{http::RobotHTTP, RemoteRobotTransport, Result};
use std::io::{stdin, stdout, Write};

fn main() -> Result<()> {
    let addr = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "localhost:1111".into());

    let robot = RobotHTTP::create(&format!("http://{addr}"))?;

    let mut inp = String::new();
    loop {
        print!("> ");
        stdout().flush()?;

        inp.clear();
        stdin().read_line(&mut inp)?;
        let inp = inp.trim();

        match inp {
            "" => continue,
            "exit" => break,
            inp => {
                let Ok(cmd) = roblib::cmd::parsing::commands::Concrete::parse_str(&mut inp.split(SEPARATOR)) else {
                    println!("Couldn't parse command");
                    continue;
                };

                print!("< ");
                match cmd {
                    Concrete::MoveRobot(c) => {
                        robot.cmd(c)?;
                    }
                    Concrete::MoveRobotByAngle(c) => {
                        robot.cmd(c)?;
                    }
                    Concrete::StopRobot(c) => {
                        robot.cmd(c)?;
                    }
                    Concrete::Led(c) => {
                        robot.cmd(c)?;
                    }
                    Concrete::ServoAbsolute(c) => {
                        robot.cmd(c)?;
                    }
                    Concrete::Buzzer(c) => {
                        robot.cmd(c)?;
                    }
                    Concrete::TrackSensor(c) => {
                        println!("{:?}", robot.cmd(c)?);
                    }
                    Concrete::UltraSensor(c) => {
                        println!("{}", robot.cmd(c)?);
                    }

                    Concrete::ReadPin(c) => {
                        println!("{}", robot.cmd(c)?);
                    }
                    Concrete::SetPin(c) => {
                        robot.cmd(c)?;
                    }
                    Concrete::SetPwm(c) => {
                        robot.cmd(c)?;
                    }
                    Concrete::ServoBasic(c) => {
                        robot.cmd(c)?;
                    }

                    Concrete::Nop(c) => {
                        robot.cmd(c)?;
                    }
                    Concrete::GetUptime(c) => {
                        println!("{:?}", robot.cmd(c)?);
                    }

                    Concrete::GetPosition(c) => {
                        if let Some(p) = robot.cmd(c)? {
                            println!("{}", p)
                        } else {
                            println!("<")
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
