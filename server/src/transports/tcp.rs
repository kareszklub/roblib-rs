use std::sync::Arc;

use crate::{cmd::execute_concrete, Backends};
use actix::spawn;
use actix_web::rt::net::{TcpListener, TcpStream};
use anyhow::Result;
use roblib::cmd::Concrete;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::ToSocketAddrs,
};

pub(crate) async fn start(addr: impl ToSocketAddrs, robot: Arc<Backends>) -> Result<()> {
    let server = TcpListener::bind(addr).await?;
    spawn(run(server, robot));
    Ok(())
}

async fn run(server: TcpListener, robot: Arc<Backends>) -> Result<()> {
    loop {
        let (stream, _addr) = server.accept().await?;
        spawn(handle_client(robot.clone(), stream));
    }
}

async fn handle_client(robot: Arc<Backends>, mut stream: TcpStream) -> Result<()> {
    let mut buf = vec![0; 512];

    loop {
        buf.resize(stream.read_u32().await? as usize, 0);

        stream.read_exact(&mut buf).await?;

        let res = match postcard::from_bytes::<Concrete>(&buf) {
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
                stream.write_all(&b).await?;
            }
            Ok(None) => (),
            Err(e) => {
                stream.write_all(e.to_string().as_bytes()).await?;
            }
        }
    }
}
