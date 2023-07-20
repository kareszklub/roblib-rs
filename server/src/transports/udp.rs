use std::{io::Cursor, sync::Arc};

use actix::spawn;
use actix_web::rt::net::UdpSocket;
use anyhow::Result;
use roblib::cmd;
use tokio::net::ToSocketAddrs;

use crate::{cmd::execute_concrete, Backends};

pub(crate) async fn start(addr: impl ToSocketAddrs, robot: Arc<Backends>) -> Result<()> {
    let server = UdpSocket::bind(addr).await?;
    spawn(run(server, robot)).await??;

    Ok(())
}

async fn run(server: UdpSocket, robot: Arc<Backends>) -> Result<()> {
    let mut buf = [0u8; 1024];

    loop {
        let (len, addr) = server.recv_from(&mut buf).await?;

        let (id, cmd): (u32, cmd::Concrete) = bincode::deserialize(&buf[..len])?;

        let mut c = Cursor::new(&mut buf[..]);
        bincode::serialize_into(&mut c, &id)?;

        let res = execute_concrete(
            cmd,
            robot.clone(),
            &mut bincode::Serializer::new(&mut c, bincode::DefaultOptions::new()),
        )
        .await?;

        if res.is_some() {
            server.send_to(&buf, addr).await?;
        }
    }
}
