#[macro_use]
extern crate log;

mod cmd;
mod logger;
mod robot;
mod ws;

use crate::cmd::Cmd;
use actix_web::{
    get,
    middleware::DefaultHeaders,
    post,
    web::{self, Payload},
    App, Error, HttpRequest, HttpResponse, HttpServer, Responder,
};
use actix_web_actors::ws::start as ws_start;

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
    let res = Cmd::exec_str(&format!("{} {}", cmd, req_body));
    HttpResponse::Ok().body(res)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    logger::init_log();
    info!("Starting server");

    HttpServer::new(|| {
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
    .bind("::1:8080")?
    .run()
    .await
}
