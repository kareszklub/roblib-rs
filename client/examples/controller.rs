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

        let res = match inp {
            "" => continue,
            "exit" => break,
            _ => {
                let Ok(cmd) = inp.parse() else {
                    println!("Couldn't parse command");
                    continue;
                };
                robot.cmd(cmd)?
            }
        };

        println!("< {res}");
    }

    Ok(())
}
