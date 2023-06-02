#[macro_use]
extern crate log;

mod logger;
mod ws;

use actix_web::{
    get,
    middleware::DefaultHeaders,
    post,
    web::{Data, Payload},
    App, Error, HttpRequest, HttpResponse, HttpServer, Responder,
};
use actix_web_actors::ws::start as ws_start;
use roblib::{cmd::Cmd, Robot};
use std::sync::Arc;

const DEFAULT_PORT: u16 = 1111;

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

/// http endpoint intended for one-time commands
/// for anything more complicated, use websockets
#[post("/cmd")]
async fn cmd_index(body: String, state: Data<AppState>) -> impl Responder {
    HttpResponse::Ok().body(Cmd::exec_str(&body, state.robot.as_ref()).await)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    logger::init_log(Some("actix_web=info,roblib_server=info,roblib=debug"));

    let port: u16 = match std::env::args().nth(1) {
        Some(s) => s.parse().expect("port must be a valid number"),
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
    info!("Server features: {}", features.join(", "));

    #[cfg(feature = "camloc")]
    let camloc_service = {
        use roblib::camloc_server::{
            extrapolations::{Extrapolation, LinearExtrapolation},
            service::LocationService,
        };
        LocationService::start(
            Some(Extrapolation::new::<LinearExtrapolation>(
                std::time::Duration::from_millis(500),
            )),
            roblib::camloc_server::camloc_common::hosts::constants::MAIN_PORT,
            None,
            std::time::Duration::from_millis(500),
        )
        .await
        .ok()
    };

    #[cfg(feature = "roland")]
    let roland = {
        match roblib::roland::GPIORoland::try_init() {
            Ok(r) => {
                info!("GPIO operational");
                info!("Server launching in production mode");
                Some(r)
            }

            Err(err) => {
                info!("Failed to initialize GPIO: {}", err);
                info!("Server launching in test mode");
                None
            }
        }
    };

    #[cfg(feature = "gpio")]
    let raw_gpio = {
        match roblib::gpio::Robot::new() {
            Ok(r) => {
                info!("GPIO operational");
                info!("Server launching in production mode");
                Some(r)
            }

            Err(err) => {
                info!("Failed to initialize GPIO: {}", err);
                info!("Server launching in test mode");
                None
            }
        }
    };

    let robot = Robot {
        #[cfg(feature = "camloc")]
        camloc_service,
        #[cfg(feature = "gpio")]
        raw_gpio,
        #[cfg(feature = "roland")]
        roland,
    }
    .into();

    let data = Data::new(AppState { robot });

    info!("Webserver starting on port {}", &port);
    HttpServer::new(move || {
        App::new()
            .wrap(
                DefaultHeaders::new()
                    .add(("Server", "roblib-rs"))
                    .add(("X-Version", env!("CARGO_PKG_VERSION"))),
            )
            .wrap(logger::actix_log())
            .app_data(data.clone())
            .service(echo)
            .service(ws_index)
            .service(cmd_index)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
