use std::{
    io::{Cursor, Write},
    net::{ToSocketAddrs, UdpSocket},
    sync::Mutex,
};

use roblib::cmd::parsing::Readable;

use crate::RemoteRobotTransport;

pub struct RobotUDP {
    stuffe: Mutex<(UdpSocket, Cursor<[u8; 1024]>)>,
}

impl RobotUDP {
    pub fn new(robot: impl ToSocketAddrs) -> anyhow::Result<Self> {
        let sock = UdpSocket::bind("0.0.0.0")?;
        sock.connect(robot)?;

        Ok(Self {
            stuffe: Mutex::new((sock, Cursor::new([0; 1024]))),
        })
    }
}

impl RemoteRobotTransport for RobotUDP {
    fn cmd<C: roblib::cmd::Command>(&self, cmd: C) -> anyhow::Result<C::Return>
    where
        C::Return: Readable,
    {
        let mut s = self.stuffe.lock().unwrap();
        let (sock, buf) = &mut *s;

        cmd.write_binary(buf as &mut dyn Write)?;
        sock.send(buf.get_ref())?;
        buf.set_position(0);

        Ok(if std::mem::size_of::<C::Return>() != 0 {
            sock.recv(buf.get_mut())?;

            Readable::parse_binary(buf)?
        } else {
            unsafe { std::mem::zeroed() }
        })
    }
}
