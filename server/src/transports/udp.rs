use std::{io::Cursor, sync::Arc};

use actix::spawn;
use actix_web::rt::net::UdpSocket;
use anyhow::Result;
use roblib::cmd::{parsing::Readable, Concrete};

use crate::{cmd::execute_concrete, Robot};

pub(crate) async fn start(port: u16, robot: Arc<Robot>) -> Result<()> {
    let server = UdpSocket::bind(("0.0.0.0", port)).await?;
    spawn(run(server, robot));

    Ok(())
}

async fn run(server: UdpSocket, robot: Arc<Robot>) -> Result<()> {
    let mut buf = [0u8; 1024];

    loop {
        let (len, addr) = server.recv_from(&mut buf).await?;

        let concrete = Concrete::parse_binary(&mut Cursor::new(&buf[..len]))?;
        let res = execute_concrete(concrete, robot.clone()).await?;

        if let Some(res) = res {
            let mut buf = Cursor::new(&mut buf[..]);
            res.write_binary(&mut buf)?;

            let res = &buf.get_ref()[..buf.position() as usize];
            server.send_to(res, addr).await?;
        }
    }
}
