#[macro_use]
extern crate log;

#[cfg(feature = "camloc")]
mod camloc;

mod cmd;
mod logger;

// mod tcp;
// mod udp;
mod ws;

use crate::cmd::execute_command_text;
use actix_web::{
    get,
    middleware::DefaultHeaders,
    post,
    web::{Data, Payload},
    App, Error, HttpRequest, HttpResponse, HttpServer, Responder,
};
use actix_web_actors::ws::start as ws_start;
use anyhow::{anyhow, Result};
use roblib::cmd::{parsing::Writable, SEPARATOR};
use std::{sync::Arc, time::Instant};

const DEFAULT_PORT: u16 = 1111;

// FIXME: REMOVE
#[allow(unused)]
pub(crate) struct Robot {
    pub startup_time: Instant,

    #[cfg(all(feature = "gpio", feature = "backend"))]
    pub raw_gpio: Option<roblib::gpio::backend::GpioBackend>,

    #[cfg(all(feature = "roland", feature = "backend"))]
    pub roland: Option<roblib::roland::backend::RolandBackend>,

    #[cfg(all(feature = "camloc"))]
    pub camloc: Option<camloc::Camloc>,
}

struct AppState {
    robot: Arc<Robot>,
}

/// websocket endpoint
/// will attempt to upgrade to websocket connection
#[get("/ws")]
async fn ws_index(
    req: HttpRequest,
    stream: Payload,
    state: Data<AppState>,
) -> Result<HttpResponse, Error> {
    ws_start(ws::WebSocket::new(state.robot.clone()), &req, stream)
}

#[post("/cmd")]
async fn cmd_index(body: String, state: Data<AppState>) -> impl Responder {
    match execute_command_text(&mut body.split(SEPARATOR), state.robot.clone()).await {
        Ok(Some(s)) => {
            let mut b = String::new();

            match Writable::write_str(&*s, &mut |r| {
                b.push_str(r);
                b.push(SEPARATOR);
            }) {
                Ok(()) => {
                    b.pop();
                    HttpResponse::Ok().body(b)
                }
                Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            }
        }
        Ok(None) => HttpResponse::Ok().into(),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[actix_web::main]
async fn main() -> Result<()> {
    logger::init_log(Some("actix_web=info,roblib_server=debug,roblib=debug"));

    let web_port: u16 = match std::env::args().nth(1) {
        Some(s) => s
            .parse()
            .map_err(|_| anyhow!("port must be a valid number"))?,
        None => DEFAULT_PORT,
    };

    info!("Server starting up");
    let features: Vec<&str> = vec![
        #[cfg(feature = "roland")]
        "roland",
        #[cfg(feature = "gpio")]
        "gpio",
        #[cfg(feature = "camloc")]
        "camloc",
    ];
    info!("Compiled with features: {}", features.join(", "));

    #[cfg(feature = "camloc")]
    let camloc = {
        use roblib::camloc::server::{extrapolations::LinearExtrapolation, service};

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

    let robot = Arc::new(Robot {
        startup_time: Instant::now(),

        #[cfg(all(feature = "roland", feature = "backend"))]
        roland,

        #[cfg(all(feature = "gpio", feature = "backend"))]
        raw_gpio,

        #[cfg(feature = "camloc")]
        camloc,
    });

    // let tcp_port = 12_345;
    // info!("TCP starting on port {tcp_port}");
    // tcp::start(tcp_port, robot.clone()).await?;

    // let udp_port = tcp_port + 1;
    // info!("UDP starting on port {udp_port}");
    // udp::start(udp_port, robot.clone()).await?;

    info!("Webserver starting on port {web_port}");

    let data = Data::new(AppState { robot });
    Ok(HttpServer::new(move || {
        App::new()
            .wrap(
                DefaultHeaders::new()
                    .add(("Server", "roblib-rs"))
                    .add(("X-Version", env!("CARGO_PKG_VERSION"))),
            )
            .wrap(logger::actix_log())
            .app_data(data.clone())
            .service(ws_index)
            .service(cmd_index)
    })
    .bind(("0.0.0.0", web_port))?
    .run()
    .await?)
}
