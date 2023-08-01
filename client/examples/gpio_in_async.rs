use roblib::{
    event::GpioPin,
    gpio::{GpioAsync, Mode, OutputPinAsync, SubscribablePinAsync, TypedGpioAsync},
};
use roblib_client::{
    async_robot::RobotAsync, logger::init_log, transports::tcp::TcpAsync, Result, Robot,
};

const IN: u8 = 2;
const OUT: u8 = 3;

// #[tokio::main(flavor = "current_thread")]
// async fn main() -> Result<()> {
//     init_log(Some("debug"));
//
//     let ip = std::env::args()
//         .nth(1)
//         .unwrap_or_else(|| "localhost:1110".into());
//
//     let robot = Box::leak(Box::new(RobotAsync::new(TcpAsync::connect(ip).await?)));
//
//     log::info!("setup pins");
//     let mut inp = TypedGpioAsync::input_pin(robot, IN).await?;
//     let mut out = TypedGpioAsync::output_pin(robot, OUT).await?;
//
//     out.set(true).await?;
//
//     log::info!("subscribe");
//     // inp.subscribe(move |b| async move {
//     //     out.set(b).await?;
//     //     dbg!(b);
//     //     Ok(())
//     // })
//     // .await?;
//     inp.subscribe(move |b| async move {
//         dbg!(b);
//         Ok(())
//     })
//     .await?;
//
//     loop {
//         std::thread::park();
//     }
// }

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    init_log(Some("debug"));

    let ip = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "localhost:1110".into());

    let robot = Box::leak(Box::new(RobotAsync::new(TcpAsync::connect(ip).await?)));

    log::info!("setup pins");
    robot.pin_mode(IN, Mode::Input).await?;
    robot.pin_mode(OUT, Mode::Output).await?;

    log::info!("subscribe");
    robot
        .subscribe(GpioPin(IN), move |b| async move {
            dbg!(b);
            Ok(())
        })
        .await?;

    loop {
        std::thread::park();
    }
}
