#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

mod robot;
mod utils;
mod ws;

use crate::utils::exec_str;
use actix_web::{
    get,
    middleware::DefaultHeaders,
    post,
    web::{self, Payload},
    App, Error, HttpRequest, HttpResponse, HttpServer, Responder,
};
use actix_web_actors::ws::start as ws_start;
use roblib_shared::logger;

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
    let res = ws_start(ws::WebSocket::new(), &req, stream);
    res
}

/// http endpoint intended for one-time commands
/// for anything more complicated, use websockets
#[post("/cmd/{cmd}")]
async fn cmd_index(req_body: String, cmd: web::Path<String>) -> impl Responder {
    let res = exec_str(&format!("{} {}", cmd, req_body));
    HttpResponse::Ok().body(res)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    logger::init_log(Some("actix_web=info,roblib_server=info"));
    info!("Starting server");

    HttpServer::new(move || {
        App::new()
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
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
