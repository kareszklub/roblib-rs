use roblib::{cmd::Concrete, text_format};
use roblib_client::{
    transports::{udp::Udp, Transport},
    Result,
};
use std::io::{stdin, stdout, Write};

fn main() -> Result<()> {
    let addr = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "localhost:1110".into());

    let robot = Udp::connect(addr)?;

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
                let Ok(cmd) = text_format::de::from_str::<Concrete>(inp) else {
                    println!("Couldn't parse command");
                    continue;
                };

                println!("{}", text_format::ser::to_string(&cmd)?);

                print!("< ");
                execute(cmd, &robot)?;
            }
        }
    }

    Ok(())
}

fn execute(cmd: Concrete, robot: &impl Transport) -> Result<()> {
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

        Concrete::Subscribe(_) => {
            println!("Subscribe no supported");
        }
        Concrete::Unsubscribe(_) => {
            println!("Unsubscribe no supported");
        }

        Concrete::Nop(c) => robot.cmd(c)?,
        Concrete::GetUptime(c) => println!("{:?}", robot.cmd(c)?),

        Concrete::GetPosition(c) => {
            if let Some(p) = robot.cmd(c)? {
                println!("{}", p)
            } else {
                println!("<")
            }
        }

        Concrete::Abort(_) => {
            println!("Abort no supported");
        }
    }
    Ok(())
}
