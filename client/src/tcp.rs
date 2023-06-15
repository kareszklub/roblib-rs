use std::{
    net::{TcpStream, ToSocketAddrs},
    sync::Mutex,
};

use roblib::cmd::parsing::Readable;

use crate::RemoteRobotTransport;

pub struct RobotTCP {
    socket: Mutex<TcpStream>,
}

impl RobotTCP {
    pub fn new(robot: impl ToSocketAddrs) -> anyhow::Result<Self> {
        Ok(Self {
            socket: TcpStream::connect(robot)?.into(),
        })
    }
}

impl RemoteRobotTransport for RobotTCP {
    fn cmd<C: roblib::cmd::Command>(&self, cmd: C) -> anyhow::Result<C::Return>
    where
        C::Return: Readable,
    {
        let mut s = self.socket.lock().unwrap();
        cmd.write_binary(&mut *s)?;

        Ok(if std::mem::size_of::<C::Return>() != 0 {
            Readable::parse_binary(&mut *s)?
        } else {
            unsafe { std::mem::zeroed() }
        })
    }
}
