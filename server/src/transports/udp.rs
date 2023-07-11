use std::sync::Arc;

use actix::spawn;
use actix_web::rt::net::UdpSocket;
use anyhow::Result;
use roblib::cmd::Concrete;

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

        let res = match postcard::from_bytes::<Concrete>(&buf[..len]) {
            Ok(c) => {
                let mut serializer = postcard::Serializer {
                    output: postcard::ser_flavors::StdVec::new(),
                };

                match execute_concrete(c, robot.clone(), &mut serializer).await {
                    Ok(r) => {
                        if let Some(()) = r {
                            postcard::ser_flavors::Flavor::finalize(serializer.output)
                                .map(Some)
                                .map_err(anyhow::Error::new)
                        } else {
                            Ok(None)
                        }
                    }
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(e.into()),
        };

        match res {
            Ok(Some(b)) => {
                server.send_to(&b, addr).await?;
            }
            Ok(None) => (),
            Err(e) => {
                server.send_to(e.to_string().as_bytes(), addr).await?;
            }
        }
    }
}
