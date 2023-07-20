#[macro_use]
extern crate log;

#[cfg(feature = "camloc")]
mod camloc;

mod cmd;
mod logger;

mod transports;
use serde::Deserialize;
use transports::udp;

use actix_web::{middleware::DefaultHeaders, web::Data, App, HttpServer};

use anyhow::Result;
use std::{sync::Arc, time::Instant};

struct Backends {
    pub startup_time: Instant,

    #[cfg(all(feature = "gpio", feature = "backend"))]
    pub raw_gpio: Option<roblib::gpio::backend::GpioBackend>,

    #[cfg(all(feature = "roland", feature = "backend"))]
    pub roland: Option<roblib::roland::backend::RolandBackend>,

    #[cfg(all(feature = "camloc"))]
    pub camloc: Option<camloc::Camloc>,
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
    logger::init_log(Some("actix_web=info,roblib_server=debug,roblib=debug"));

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

    #[cfg(feature = "camloc")]
    let camloc = {
        use roblib::camloc::server::{extrapolations::LinearExtrapolation, service};

        // TODO: config
        let serv = service::start(
            Some(LinearExtrapolation::new()),
            roblib::camloc::MAIN_PORT,
            None,
            std::time::Duration::from_millis(500),
        )
        .await;

        match serv {
            Ok(s) => {
                info!("Camloc operational");
                Some(camloc::Camloc::new(s))
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
        match roblib::gpio::backend::GpioBackend::new() {
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

        #[cfg(all(feature = "roland", feature = "backend"))]
        roland,

        #[cfg(all(feature = "gpio", feature = "backend"))]
        raw_gpio,

        #[cfg(feature = "camloc")]
        camloc,
    });

    // info!("TCP starting on port {tcp_port}");
    // tcp::start((tcp_host, tcp_port), robot.clone()).await?;

    info!("UDP starting on port {udp_port}");
    udp::start((udp_host, udp_port), robot.clone()).await?;

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
