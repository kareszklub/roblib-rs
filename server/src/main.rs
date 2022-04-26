#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

mod robot;
mod utils;
mod ws;

use crate::{
    robot::{init_robot, Robot},
    utils::exec_str,
};
use actix_web::{
    get,
    middleware::DefaultHeaders,
    post,
    web::{self, Payload},
    App, Error, HttpRequest, HttpResponse, HttpServer, Responder,
};
use actix_web_actors::ws::start as ws_start;
use roblib_shared::logger;
use rppal::gpio::Error as GpioError;

lazy_static! {
    static ref ROBOT: (Robot, Option<GpioError>) = init_robot();
}

const DEFAULT_PORT: u16 = 1111;

struct AppState {
    robot: &'static Robot,
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
    data: web::Data<AppState>,
    req: HttpRequest,
    stream: Payload,
) -> Result<HttpResponse, Error> {
    let res = ws_start(ws::WebSocket::new(data.robot.clone()), &req, stream);
    res
}

/// http endpoint intended for one-time commands
/// for anything more complicated, use websockets
#[post("/cmd")]
async fn cmd_index(data: web::Data<AppState>, req_body: String) -> impl Responder {
    let res = exec_str(&req_body, &data.robot);
    HttpResponse::Ok().body(res)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    logger::init_log(Some("actix_web=info,roblib_server=info"));

    let port: u16 = match std::env::args().collect::<Vec<String>>().get(1) {
        Some(s) => s.parse().expect("port must be a valid number"),
        None => DEFAULT_PORT,
    };

    info!("Starting server on port {}", &port);

    if let Some(err) = &ROBOT.1 {
        info!("Failed to initialize GPIO: {}", err);
        info!("Server launching in test mode");
    } else {
        info!("GPIO operational");
        info!("Server launching in production mode");
    }

    HttpServer::new(move || {
        App::new()
            .data(AppState { robot: &ROBOT.0 })
            .wrap(
                DefaultHeaders::new()
                    .header("Server", "roblib-rs")
                    .header("X-Version", env!("CARGO_PKG_VERSION")),
            )
            .wrap(logger::actix_log())
            .service(hello)
            .service(echo)
            .service(ws_index)
            .service(cmd_index)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
