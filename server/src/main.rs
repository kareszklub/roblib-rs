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
use roblib::{
    camloc_server::{
        extrapolations::{Extrapolation, LinearExtrapolation},
        service::LocationService,
    },
    cmd::Cmd,
    gpio::roland::GPIORoland,
};
use std::{sync::Arc, time::Duration};

const DEFAULT_PORT: u16 = 1111;

struct AppState {
    roland: Arc<Option<GPIORoland>>,
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

/// websocket endpoint
/// will attempt to upgrade to websocket connection
#[get("/ws")]
async fn ws_index(
    req: HttpRequest,
    stream: Payload,
    state: Data<AppState>,
) -> Result<HttpResponse, Error> {
    ws_start(ws::WebSocket::new(state.roland.clone()), &req, stream)
}

/// http endpoint intended for one-time commands
/// for anything more complicated, use websockets
#[post("/cmd")]
async fn cmd_index(body: String, state: Data<AppState>) -> impl Responder {
    HttpResponse::Ok().body(Cmd::exec_str(&body, state.roland.as_ref().as_ref()))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    logger::init_log(Some("actix_web=info,roblib_server=info,roblib=debug"));

    let port: u16 = match std::env::args().nth(1) {
        Some(s) => s.parse().expect("port must be a valid number"),
        None => DEFAULT_PORT,
    };

    info!("Starting server on port {}", &port);
    info!(
        "Server edition: {}",
        if cfg!(feature = "roland") {
            "Roland"
        } else {
            "Generic pin commands only"
        }
    );

    let camloc_service = LocationService::start(
        Some(Extrapolation::new::<LinearExtrapolation>(
            Duration::from_millis(500),
        )),
        roblib::camloc_server::camloc_common::hosts::constants::MAIN_PORT,
        None,
        Duration::from_millis(500),
    )
    .await
    .ok();

    let roland = match GPIORoland::try_init(camloc_service) {
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
    .into();

    let data = Data::new(AppState { roland });
    HttpServer::new(move || {
        App::new()
            .wrap(
                DefaultHeaders::new()
                    .add(("Server", "roblib-rs"))
                    .add(("X-Version", env!("CARGO_PKG_VERSION"))),
            )
            .wrap(logger::actix_log())
            .app_data(data.clone())
            .service(hello)
            .service(echo)
            .service(ws_index)
            .service(cmd_index)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
