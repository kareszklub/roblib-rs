use std::io::{Cursor, Write};

use roblib::cmd::{
    has_return,
    parsing::{Readable, Writable},
    Command, Concrete,
};

use crate::Transport;

pub struct Udp {
    stuffe: std::sync::Mutex<(std::net::UdpSocket, Cursor<[u8; 1024]>)>,
}

impl Udp {
    pub fn new(robot: impl std::net::ToSocketAddrs) -> anyhow::Result<Self> {
        let sock = std::net::UdpSocket::bind("0.0.0.0")?;
        sock.connect(robot)?;

        Ok(Self {
            stuffe: (sock, Cursor::new([0; 1024])).into(),
        })
    }
}

impl Transport for Udp {
    fn cmd<C: Command>(&self, cmd: C) -> anyhow::Result<C::Return>
    where
        C::Return: Readable,
    {
        let concrete: Concrete = cmd.into();

        let mut s = self.stuffe.lock().unwrap();
        let (sock, buf) = &mut *s;

        buf.set_position(0);
        concrete.write_binary(buf as &mut dyn Write)?;
        sock.send(&buf.get_ref()[..buf.position() as usize])?;

        Ok(if has_return::<C>() {
            buf.set_position(0);
            sock.recv(buf.get_mut())?;

            Readable::parse_binary(buf)?
        } else {
            unsafe { std::mem::zeroed() }
        })
    }
}

#[cfg(feature = "async")]
pub struct UdpAsync {
    stuffe: tokio::sync::Mutex<(tokio::net::UdpSocket, Cursor<[u8; 1024]>)>,
}

#[cfg(feature = "async")]
impl UdpAsync {
    pub async fn new(robot: impl tokio::net::ToSocketAddrs) -> anyhow::Result<Self> {
        let sock = tokio::net::UdpSocket::bind("0.0.0.0:0").await?;
        sock.connect(robot).await?;

        Ok(Self {
            stuffe: (sock, Cursor::new([0; 1024])).into(),
        })
    }
}

#[cfg(feature = "async")]
#[cfg_attr(feature = "async", async_trait::async_trait)]
impl super::TransportAsync for UdpAsync {
    async fn cmd_async<C: Command + Send>(&self, cmd: C) -> anyhow::Result<C::Return>
    where
        C::Return: Readable + Send,
    {
        let concrete: Concrete = cmd.into();

        let mut s = self.stuffe.lock().await;
        let (sock, buf) = &mut *s;

        buf.set_position(0);
        concrete.write_binary(buf as &mut dyn Write)?;
        sock.send(&buf.get_ref()[..buf.position() as usize]).await?;

        Ok(if has_return::<C>() {
            buf.set_position(0);
            sock.recv(buf.get_mut()).await?;

            Readable::parse_binary(buf)?
        } else {
            unsafe { std::mem::zeroed() }
        })
    }
}
