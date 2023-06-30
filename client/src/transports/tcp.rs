use roblib::{
    cmd::{has_return, Command, Concrete},
    Readable, Writable,
};

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

impl super::Transport for Tcp {
    fn cmd<C: Command>(&self, cmd: C) -> anyhow::Result<C::Return>
    where
        C::Return: Readable,
    {
        let mut s = self.socket.lock().unwrap();

        Into::<Concrete>::into(cmd).write_binary(&mut *s)?;

        Ok(if has_return::<C>() {
            Readable::parse_binary(&mut *s)?
        } else {
            unsafe { std::mem::zeroed() }
        })
    }
}

#[cfg(feature = "async")]
pub struct TcpAsync {
    socket: futures_util::lock::Mutex<tokio_util::compat::Compat<tokio::net::TcpStream>>,
}

#[cfg(feature = "async")]
impl TcpAsync {
    pub async fn new(robot: impl tokio::net::ToSocketAddrs) -> anyhow::Result<Self> {
        use tokio_util::compat::TokioAsyncReadCompatExt;

        Ok(Self {
            socket: tokio::net::TcpStream::connect(robot).await?.compat().into(),
        })
    }
}

#[cfg(feature = "async")]
#[cfg_attr(feature = "async", async_trait::async_trait)]
impl super::TransportAsync for TcpAsync {
    async fn cmd_async<C: Command + Send>(&self, cmd: C) -> anyhow::Result<C::Return>
    where
        C::Return: Readable + Send,
    {
        use tokio::io::AsyncWriteExt;

        let mut buf = std::io::Cursor::new([0u8; 512]);
        Into::<Concrete>::into(cmd).write_binary(&mut buf)?;

        self.socket
            .lock()
            .await
            .get_mut()
            .write_all(&buf.get_ref()[..(buf.position() as usize)])
            .await?;

        Ok(if has_return::<C>() {
            let mut t = self.socket.lock().await;
            Readable::parse_binary_async(&mut *t).await?
        } else {
            unsafe { std::mem::zeroed() }
        })
    }
}
