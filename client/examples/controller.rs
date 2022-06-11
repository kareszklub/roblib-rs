use roblib_client::{http::Robot, Result};
use std::io::Write;

#[roblib_client::main]
async fn main() -> Result<()> {
    let ip = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "localhost:1111".into());
    let robot = Robot::new(&format!("http://{ip}"));

    loop {
        let mut inp = String::new();
        print!("> ");
        std::io::stdout().flush().unwrap();
        std::io::stdin().read_line(&mut inp).unwrap();
        let inp = inp.trim();
        if inp.is_empty() {
            continue;
        }
        if inp == "exit" {
            break;
        }
        let res = robot.send(inp.into()).await?;
        println!("< {res}");
    }

    Ok(())
}
