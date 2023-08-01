use super::ws::{ws_route, Rx};
use crate::{cmd::execute_concrete, Backends};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Router, Server,
};
use roblib::{cmd, text_format};
use std::{net::SocketAddr, sync::Arc};
use tokio::{
    net::{lookup_host, ToSocketAddrs},
    task::JoinHandle,
};

pub(crate) type AppState = State<(Arc<Backends>, Arc<Rx>)>;

pub(crate) async fn start(
    addr: impl ToSocketAddrs,
    robot: Arc<Backends>,
    rx: Rx,
) -> JoinHandle<Result<(), anyhow::Error>> {
    let abort = robot.abort_token.clone();
    let app = Router::new()
        .route("/", get(index))
        .route("/cmd", post(cmd))
        .route("/ws", get(ws_route))
        .with_state((robot, Arc::new(rx)));

    let addr = lookup_host(&addr).await.unwrap().next().unwrap();
    tokio::spawn(async move {
        Server::bind(&addr)
            .serve(app.into_make_service_with_connect_info::<SocketAddr>())
            .with_graceful_shutdown(async {
                abort.cancelled().await;
                log::debug!("abort: http")
            })
            .await?;
        Ok(())
    })
}

async fn cmd(State((robot, _)): AppState, body: String) -> Result<impl IntoResponse, Response> {
    let Ok(cmd) = text_format::de::from_str::<cmd::Concrete>(&body) else {
        return Err((StatusCode::BAD_REQUEST, "invalid cmd").into_response());
    };

    let mut buf = String::new();
    if let Err(e) =
        execute_concrete(cmd, robot, &mut text_format::ser::Serializer::new(&mut buf)).await
    {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response());
    }

    Ok(buf)
}

// redirect to GitHub repo for no particular reason
async fn index() -> (StatusCode, Redirect) {
    (
        StatusCode::FOUND,
        Redirect::temporary("https://github.com/kareszklub/roblib-rs"),
    )
}
