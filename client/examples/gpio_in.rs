use roblib_client::{logger::init_log, Result, Robot};

const IN: u8 = 2;
const OUT: u8 = 3;

// fn main() -> Result<()> {
//     use roblib::gpio::{Gpio, Mode};
//     use roblib_client::transports::udp::Udp;
//     init_log(Some("debug"));
//
//     let ip = std::env::args()
//         .nth(1)
//         .unwrap_or_else(|| "localhost:1110".into());
//
//     let robot = Box::leak(Box::new(Robot::new(Udp::connect(ip)?)));
//
//     log::info!("set pin mode");
//     robot.pin_mode(OUT, Mode::Output)?;
//     robot.pin_mode(IN, Mode::Input)?;
//     log::info!("subscribe");
//     robot.subscribe(roblib::gpio::event::GpioPin(IN), |p| {
//         dbg!(p);
//         robot.write_pin(OUT, p)?;
//         Ok(())
//     })?;
//     loop {
//         std::thread::park();
//     }
// }

fn main() -> Result<()> {
    use roblib::gpio::{OutputPin, SubscribablePin, TypedGpio};
    use roblib_client::transports::tcp::Tcp;
    init_log(Some("debug"));

    let ip = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "localhost:1110".into());

    let robot = Box::leak(Box::new(Robot::new(Tcp::connect(ip)?)));

    log::info!("setup pins");
    let mut inp = TypedGpio::input_pin(robot, IN)?;
    let mut out = TypedGpio::output_pin(robot, OUT)?;

    log::info!("subscribe");
    inp.subscribe(move |b| {
        out.set(b)?;
        dbg!(b);
        Ok(())
    })?;

    loop {
        std::thread::park();
    }
}
