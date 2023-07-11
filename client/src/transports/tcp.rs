use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::io::{Read, Write};

pub struct Tcp {
    socket: std::sync::Mutex<std::net::TcpStream>,
}

impl Tcp {
    pub fn connect(robot: impl std::net::ToSocketAddrs) -> anyhow::Result<Self> {
        Ok(Self {
            socket: std::net::TcpStream::connect(robot)?.into(),
        })
    }
}

struct Sock<'a> {
    sock: &'a mut std::net::TcpStream,
    buff: &'a mut [u8],
}

impl<'a> Sock<'a> {
    fn new(sock: &'a mut std::net::TcpStream, buff: &'a mut [u8]) -> Self {
        Self { sock, buff }
    }
}

impl<'de> postcard::de_flavors::Flavor<'de> for Sock<'de> {
    type Remainder = ();
    type Source = ();

    fn pop(&mut self) -> postcard::Result<u8> {
        let mut b = [0];
        match self.sock.read_exact(&mut b) {
            Ok(_) => Ok(b[0]),
            Err(_) => Err(postcard::Error::DeserializeUnexpectedEnd),
        }
    }

    fn try_take_n(&mut self, ct: usize) -> postcard::Result<&'de [u8]> {
        match self.sock.read_exact(&mut self.buff[..ct]) {
            Ok(_) => Ok(&self.buff[..ct]),
            Err(_) => Err(postcard::Error::DeserializeUnexpectedEnd),
        }
    }

    fn finalize(self) -> postcard::Result<Self::Remainder> {
        Ok(())
    }
}

impl<'de> postcard::ser_flavors::Flavor for Sock<'de> {
    type Output = ();

    fn try_push(&mut self, data: u8) -> postcard::Result<()> {
        match self.sock.write_all(&[data]) {
            Ok(_) => Ok(()),
            Err(_) => Err(postcard::Error::SerdeSerCustom),
        }
    }

    fn finalize(self) -> postcard::Result<Self::Output> {
        Ok(())
    }
}

impl super::Transport for Tcp {
    fn send<S: serde::Serialize>(&self, value: S) -> anyhow::Result<()> {
        let mut s = self.socket.lock().unwrap();
        s.write_all(&postcard::to_slice(&value, &mut [0; 512])?)?;
        Ok(())
    }

    fn recv<D: DeserializeOwned>(&self) -> anyhow::Result<D> {
        let mut s = self.socket.lock().unwrap();
        let mut binding = [0; 512];
        let mut de = postcard::Deserializer::from_flavor(Sock::new(&mut *s, &mut binding));

        Ok(Deserialize::deserialize(&mut de)?)
    }
}

#[cfg(feature = "async")]
pub struct TcpAsync {
    socket: futures_util::lock::Mutex<tokio::net::TcpStream>,
}

#[cfg(feature = "async")]
impl TcpAsync {
    pub async fn new(robot: impl tokio::net::ToSocketAddrs) -> anyhow::Result<Self> {
        Ok(Self {
            socket: tokio::net::TcpStream::connect(robot).await?.into(),
        })
    }
}

#[cfg(feature = "async")]
#[cfg_attr(feature = "async", async_trait::async_trait)]
impl<'r> super::TransportAsync<'r> for TcpAsync {
    async fn send<S: Serialize + Send>(&self, value: S) -> anyhow::Result<()> {
        todo!()
    }
    async fn recv<D: DeserializeOwned + Send>(&self) -> anyhow::Result<D> {
        todo!()
    }
}
