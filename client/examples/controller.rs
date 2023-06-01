use roblib_client::{http::RobotHTTP, Result};
use std::io::{stdin, stdout, Write};

#[roblib_client::main]
async fn main() -> Result<()> {
    let ip = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "localhost:1111".into());
    let robot = RobotHTTP::connect(&format!("http://{ip}")).await?;

    let mut inp = String::new();
    loop {
        print!("> ");
        stdout().flush()?;

        stdin().read_line(&mut inp)?;
        let inp = inp.trim();

        let res = match inp {
            "" => continue,
            "exit" => break,
            _ => robot.send(inp.into()).await?,
        };

        println!("< {res}");
    }

    Ok(())
}
