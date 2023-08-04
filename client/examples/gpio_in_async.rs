use roblib::gpio::{OutputPinAsync, SubscribablePinAsync, TypedGpioAsync};
use roblib_client::{async_robot::RobotAsync, logger::init_log, transports::tcp::TcpAsync, Result};

const IN: u8 = 2;
const OUT: u8 = 3;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    init_log(Some("debug"));

    let ip = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "localhost:1111".into());

    let robot = RobotAsync::new(TcpAsync::connect(&ip).await?);

    log::info!("setup pins");
    let inp = robot.input_pin(IN).await?;
    let mut out = robot.output_pin(OUT).await?;

    log::info!("subscribe");
    let mut rx = inp.subscribe().await?;

    loop {
        let b = rx.recv().await?;
        dbg!(&b);
        out.set(b).await?;
    }
}
