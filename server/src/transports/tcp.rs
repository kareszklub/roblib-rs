use std::sync::Arc;

use crate::{cmd::execute_concrete, Robot};
use actix::spawn;
use actix_web::rt::net::{TcpListener, TcpStream};
use anyhow::Result;
use roblib::{cmd::Concrete, Readable, Writable};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

pub(crate) async fn start(port: u16, robot: Arc<Robot>) -> Result<()> {
    let server = TcpListener::bind(("0.0.0.0", port)).await?;
    spawn(run(server, robot));
    Ok(())
}

async fn run(server: TcpListener, robot: Arc<Robot>) -> Result<()> {
    loop {
        let (stream, _addr) = server.accept().await?;
        spawn(handle_client(robot.clone(), stream));
    }
}

async fn handle_client(robot: Arc<Robot>, mut stream: TcpStream) -> Result<()> {
    let (rx, tx) = stream.split();
    let mut rx = rx.compat();
    let mut tx = tx.compat_write();

    loop {
        let concrete = Concrete::parse_binary_async(&mut rx).await?;

        if let Some(res) = execute_concrete(concrete, robot.clone()).await? {
            Writable::write_binary_async(&*res, &mut tx).await?
        }
    }
}
