#[macro_use]
extern crate log;

mod cmd;
mod event_bus;
mod logger;
mod transports;
use anyhow::Result;
use serde::Deserialize;
use std::{
    sync::{atomic::AtomicBool, Arc},
    time::Instant,
};
use tokio::sync::{broadcast, mpsc};
use tokio_util::sync::CancellationToken;
use transports::udp;

struct Backends {
    pub startup_time: Instant,

    abort_token: CancellationToken,

    sub: event_bus::sub::Tx,

    #[cfg(all(feature = "gpio", feature = "backend"))]
    pub raw_gpio: Option<roblib::gpio::backend::SimpleGpioBackend>,

    #[cfg(all(feature = "roland", feature = "backend"))]
    pub roland: Option<roblib::roland::backend::RolandBackend>,

    #[cfg(all(feature = "camloc", feature = "backend"))]
    pub camloc: Option<roblib::camloc::server::service::LocationServiceHandle>,
}

fn def_host() -> String {
    String::from("0.0.0.0")
}
fn def_tcp_port() -> u16 {
    1110
}
fn def_udp_port() -> u16 {
    def_tcp_port()
}
fn def_web_port() -> u16 {
    def_tcp_port() + 1
}

#[derive(Debug, Deserialize)]
struct Config {
    #[serde(default = "def_host")]
    _tcp_host: String,

    #[serde(default = "def_host")]
    udp_host: String,

    #[serde(default = "def_host")]
    _web_host: String,

    #[serde(default = "def_tcp_port")]
    _tcp_port: u16,

    #[serde(default = "def_udp_port")]
    udp_port: u16,

    #[serde(default = "def_web_port")]
    _web_port: u16,
}

async fn try_main() -> Result<()> {
    logger::init_log(Some("actix_web=info,roblib_server=debug,roblib=debug"));

    let Config {
        _tcp_host,
        udp_host,
        _web_host,
        _tcp_port,
        udp_port,
        _web_port,
    } = match envy::from_env::<Config>() {
        Ok(config) => config,
        Err(error) => panic!("{:#?}", error),
    };

    info!("Server starting up");
    let features: &[&str] = &[
        #[cfg(feature = "roland")]
        "roland",
        #[cfg(feature = "gpio")]
        "gpio",
        #[cfg(feature = "camloc")]
        "camloc",
        #[cfg(feature = "backend")]
        "backend",
    ];
    info!("Compiled with features: {features:?}");

    // let event_bus = event_bus::init();
    let (udp_tx, udp_rx) = mpsc::unbounded_channel();

    #[cfg(feature = "camloc")]
    let camloc = {
        use roblib::camloc::server::{extrapolations::LinearExtrapolation, service};

        // TODO: config
        let serv = service::start(
            Some(Box::new(LinearExtrapolation::new())),
            roblib::camloc::MAIN_PORT,
            vec![],
            15.,
            std::time::Duration::from_millis(500),
        )
        .await;

        match serv {
            Ok(s) => {
                // TODO:
                // struct MySub(event_bus::Tx);
                // impl event::Subscriber for MySub {
                //     fn handle_event(&mut self, event: service::Event) {
                //         let ev: (ConcreteType, ConcreteValue) = match event {
                //             service::Event::Connect(a, c) => (
                //                 ConcreteType::CamlocConnect(event::CamlocConnect),
                //                 ConcreteValue::CamlocConnect((a, c)),
                //             ),
                //
                //             service::Event::Disconnect(a) => (
                //                 ConcreteType::CamlocDisconnect(event::CamlocDisconnect),
                //                 ConcreteValue::CamlocDisconnect(a),
                //             ),
                //             service::Event::PositionUpdate(p) => (
                //                 ConcreteType::CamlocPosition(event::CamlocPosition),
                //                 ConcreteValue::CamlocPosition(p),
                //             ),
                //             service::Event::InfoUpdate(a, i) => (
                //                 ConcreteType::CamlocInfoUpdate(event::CamlocInfoUpdate),
                //                 ConcreteValue::CamlocInfoUpdate((a, i)),
                //             ),
                //         };
                //
                //         if let Err(e) = self.0.send(ev) {
                //             log::error!("Camloc: event bus error: {e}");
                //         }
                //     }
                // }
                // s.subscribe(MySub(event_bus.bus_udp.clone())).await;

                info!("Camloc operational");
                Some(s)
            }

            Err(err) => {
                info!("Failed to initialize camloc: {err}");
                None
            }
        }
    };

    #[cfg(all(feature = "roland", feature = "backend"))]
    let roland = {
        match roblib::roland::backend::RolandBackend::try_init() {
            Ok(r) => {
                info!("Roland operational");
                Some(r)
            }

            Err(err) => {
                info!("Failed to initialize roland: {err}");
                None
            }
        }
    };

    #[cfg(all(feature = "gpio", feature = "backend"))]
    let raw_gpio = {
        match roblib::gpio::backend::SimpleGpioBackend::new() {
            Ok(r) => {
                info!("GPIO operational");
                Some(r)
            }

            Err(err) => {
                info!("Failed to initialize GPIO: {err}");
                None
            }
        }
    };

    let robot = Arc::new(Backends {
        startup_time: Instant::now(),

        abort_token: CancellationToken::new(),

        sub: broadcast::channel(64).0,

        #[cfg(all(feature = "roland", feature = "backend"))]
        roland,

        #[cfg(all(feature = "gpio", feature = "backend"))]
        raw_gpio,

        #[cfg(all(feature = "camloc", feature = "backend"))]
        camloc,
    });

    // info!("TCP starting on {tcp_host}:{tcp_port}");
    // tcp::start((tcp_host, tcp_port), robot.clone(), tcp_rx).await?;

    info!("UDP starting on {udp_host}:{udp_port}");
    let (udp_handle, udp_event_handle) =
        udp::start((udp_host, udp_port), robot.clone(), udp_rx).await?;

    // info!("Webserver starting on port {web_port}");
    // let data = Data::new(robot);
    // Ok(HttpServer::new(move || {
    //     App::new()
    //         .wrap(
    //             DefaultHeaders::new()
    //                 .add(("Server", format!("roblib-rs/{}", env!("CARGO_PKG_VERSION")))),
    //         )
    //         .wrap(logger::actix_log())
    //         .app_data(data.clone())
    //         .service(http::post_cmd)
    //         .service(http::index)
    //         .service(ws::index)
    // })
    // .bind((web_host, web_port))?
    // .run()
    // .await?)

    // let event_bus_handle = tokio::spawn(event_bus::connect(event_bus::EventBus::new(
    //     robot.clone(),
    //     udp_tx,
    // )));

    let ebus_handle = tokio::spawn(event_bus::init(robot.clone(), udp_tx));

    tokio::select! {
        _ = robot.abort_token.cancelled() => (),
        _ = tokio::signal::ctrl_c() => {
            log::debug!("SIGINT received");
            robot.abort_token.cancel();
        }
    };

    log::debug!("abort: main");

    udp_handle.abort();
    udp_event_handle.abort();
    let _ = ebus_handle.await;

    Ok(())
}

#[actix_web::main]
async fn main() {
    match try_main().await {
        Ok(_) => eprintln!("Bye!"),
        Err(e) => {
            eprintln!("ERROR: {e}");
            std::process::exit(1);
        }
    }
}
