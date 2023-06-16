use std::sync::Arc;

use actix_web::rt::net::TcpListener;
use anyhow::Result;

use crate::Robot;

async fn run(server: TcpListener, robot: Arc<Robot>) -> Result<()> {
    loop {
        let conn = server.accept().await?;
    }
}

pub(crate) async fn start(port: u16, robot: Arc<Robot>) -> Result<()> {
    let server = TcpListener::bind(("0.0.0.0", port)).await?;
    actix_web::rt::spawn(run(server, robot));
    Ok(())
}
