#[macro_use]
extern crate log;

mod cmd;
mod event_bus;
mod logger;
mod transports;
use anyhow::Result;
use futures_util::future::join_all;
use serde::Deserialize;
use std::{sync::Arc, time::Instant};
use tokio::sync::{broadcast, mpsc};
use tokio_util::sync::CancellationToken;
use transports::{http, tcp, udp};

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
    "0.0.0.0".into()
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
    tcp_host: String,

    #[serde(default = "def_host")]
    udp_host: String,

    #[serde(default = "def_host")]
    web_host: String,

    #[serde(default = "def_tcp_port")]
    tcp_port: u16,

    #[serde(default = "def_udp_port")]
    udp_port: u16,

    #[serde(default = "def_web_port")]
    web_port: u16,
}

async fn try_main() -> Result<()> {
    let Config {
        tcp_host,
        udp_host,
        web_host,
        tcp_port,
        udp_port,
        web_port,
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
    let (tcp_tx, tcp_rx) = broadcast::channel(1024);
    let (udp_tx, udp_rx) = mpsc::unbounded_channel();
    let (ws_tx, ws_rx) = broadcast::channel(1024);

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

    info!("TCP starting on {tcp_host}:{tcp_port}");
    let tcp_handle = tcp::start((tcp_host, tcp_port), robot.clone(), tcp_rx).await?;

    info!("UDP starting on {udp_host}:{udp_port}");
    let (udp_handle, udp_event_handle) =
        udp::start((udp_host, udp_port), robot.clone(), udp_rx).await?;

    info!("Webserver starting on port {web_port}");
    let http_handle = http::start((web_host, web_port), robot.clone(), ws_rx).await;

    let ebus_handle = tokio::spawn(event_bus::init(robot.clone(), tcp_tx, udp_tx, ws_tx));

    let mut sighandler = SigHandler::new();
    tokio::select! {
        _ = robot.abort_token.cancelled() => {
            log::error!("Abort requested internally, cleaning up...");
        },
        s = sighandler.wait() => {
            log::error!("{s} received, cleaning up...");
            robot.abort_token.cancel();
        }
    };

    log::debug!("abort: main");

    let force_stop = tokio::spawn(async move {
        sighandler.wait().await;
        log::warn!("Press ^C again to force exit (THE ROBOT WILL ESCAPE)");
        sighandler.wait().await;
        log::error!("Bye! (Force shutdown)");
        std::process::exit(1);
    });

    udp_handle.abort();
    udp_event_handle.abort();

    let mut futures = vec![http_handle, ebus_handle];
    if let Ok(mut tcp_handles) = tcp_handle.await {
        futures.append(&mut tcp_handles);
    }
    log::debug!("Waiting on {} tasks", futures.len());
    join_all(futures).await;

    force_stop.abort();
    let _ = force_stop.await;
    Ok(())
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    logger::init_log(Some("actix_web=info,roblib_server=debug,roblib=debug"));

    match try_main().await {
        Ok(_) => log::info!("Bye!"),
        Err(e) => {
            log::error!("ERROR: {e}");
            std::process::exit(1);
        }
    }
}

struct SigHandler {
    #[cfg(unix)]
    sigterm: tokio::signal::unix::Signal,
}
impl SigHandler {
    pub fn new() -> Self {
        Self {
            #[cfg(unix)]
            sigterm: tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .unwrap(),
        }
    }
    pub async fn wait(&mut self) -> &str {
        #[cfg(unix)]
        tokio::select! {
            _ = self.sigterm.recv() => "SIGTERM",
            r = tokio::signal::ctrl_c() => { r.expect("failed to listen to ctrl-c"); "SIGINT" },
        }

        #[cfg(not(unix))]
        {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to listen to ctrl-c");
            "SIGINT"
        }
    }
}
