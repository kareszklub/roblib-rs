use roblib::cmd::Concrete;
use roblib_client::{
    transports::{tcp::Tcp, Transport},
    Result,
};
use std::io::{stdin, stdout, Write};

fn main() -> Result<()> {
    let addr = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "localhost:1111".into());

    let robot = Tcp::connect(addr)?;

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
                let Ok(cmd) = roblib::text_format::de::from_str(inp) else {
                    println!("Couldn't parse command");
                    continue;
                };

                print!("< ");
                match cmd {
                    Concrete::MoveRobot(c) => robot.cmd(c)?,
                    Concrete::MoveRobotByAngle(c) => robot.cmd(c)?,
                    Concrete::StopRobot(c) => robot.cmd(c)?,
                    Concrete::Led(c) => robot.cmd(c)?,
                    Concrete::RolandServo(c) => robot.cmd(c)?,
                    Concrete::Buzzer(c) => robot.cmd(c)?,
                    Concrete::TrackSensor(c) => println!("{:?}", robot.cmd(c)?),
                    Concrete::UltraSensor(c) => println!("{}", robot.cmd(c)?),

                    Concrete::PinMode(c) => robot.cmd(c)?,
                    Concrete::ReadPin(c) => println!("{}", robot.cmd(c)?),
                    Concrete::WritePin(c) => robot.cmd(c)?,
                    Concrete::Pwm(c) => robot.cmd(c)?,
                    Concrete::Servo(c) => robot.cmd(c)?,

                    Concrete::Subscribe(c) => robot.cmd(c)?,
                    Concrete::Unsubscribe(c) => robot.cmd(c)?,
                    Concrete::Nop(c) => robot.cmd(c)?,
                    Concrete::GetUptime(c) => println!("{:?}", robot.cmd(c)?),

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
