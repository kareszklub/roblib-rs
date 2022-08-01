#[macro_use]
extern crate log;

mod logger;
mod ws;

use actix_web::{
    get, middleware::DefaultHeaders, post, web::Payload, App, Error, HttpRequest, HttpResponse,
    HttpServer, Responder,
};
use actix_web_actors::ws::start as ws_start;
use roblib::{cmd::Cmd, gpio::roland};

const DEFAULT_PORT: u16 = 1111;

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
async fn ws_index(req: HttpRequest, stream: Payload) -> Result<HttpResponse, Error> {
    ws_start(ws::WebSocket::new(), &req, stream)
}

/// http endpoint intended for one-time commands
/// for anything more complicated, use websockets
#[post("/cmd")]
async fn cmd_index(body: String) -> impl Responder {
    HttpResponse::Ok().body(Cmd::exec_str(&body))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    logger::init_log(Some("actix_web=info,roblib_server=info"));

    let port: u16 = match std::env::args().collect::<Vec<String>>().get(1) {
        Some(s) => s.parse().expect("port must be a valid number"),
        None => DEFAULT_PORT,
    };

    info!("Starting server on port {}", &port);

    match roland::try_init() {
        Ok(_) => {
            info!("GPIO operational");
            info!("Server launching in production mode");
        }

        Err(err) => {
            info!("Failed to initialize GPIO: {}", err);
            info!("Server launching in test mode");
        }
    }

    HttpServer::new(move || {
        App::new()
            .wrap(
                DefaultHeaders::new()
                    .add(("Server", "roblib-rs"))
                    .add(("X-Version", env!("CARGO_PKG_VERSION"))),
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
