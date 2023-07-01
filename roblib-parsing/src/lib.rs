#[cfg(feature = "async")]
use futures::{AsyncReadExt, AsyncWriteExt};

use std::{
    net::{IpAddr, SocketAddr},
    time::Duration,
};

pub const SEPARATOR: char = ' ';

#[cfg_attr(feature = "async", async_trait::async_trait)]
pub trait Readable: Sized {
    fn parse_text<'a>(s: &mut impl Iterator<Item = &'a str>) -> anyhow::Result<Self>;
    fn parse_binary(r: &mut impl std::io::Read) -> anyhow::Result<Self>;

    #[cfg(feature = "async")]
    async fn parse_binary_async(
        r: &mut (impl futures::AsyncRead + Send + Unpin),
    ) -> anyhow::Result<Self>
    where
        Self: Send;
}

#[cfg_attr(feature = "async", async_trait::async_trait)]
pub trait Writable {
    fn write_text(&self, w: &mut dyn std::fmt::Write) -> std::fmt::Result;
    fn write_binary(&self, w: &mut dyn std::io::Write) -> anyhow::Result<()>;

    #[cfg(feature = "async")]
    async fn write_binary_async(
        &self,
        w: &mut (dyn futures::AsyncWrite + Send + Unpin),
    ) -> anyhow::Result<()>
    where
        Self: Sync;
}

#[cfg_attr(feature = "async", async_trait::async_trait)]
impl Readable for String {
    fn parse_text<'a>(s: &mut impl Iterator<Item = &'a str>) -> anyhow::Result<Self> {
        let Some(s) = s.next() else {
            return Err(anyhow::Error::msg("Not enough arguments"))
        };
        Ok(s.to_string())
    }
    fn parse_binary(r: &mut impl std::io::Read) -> anyhow::Result<Self> {
        let mut l = [0; std::mem::size_of::<u16>()];
        r.read_exact(&mut l)?;
        let l = u16::from_be_bytes(l);

        let mut v = vec![0; l as usize];
        r.read_exact(&mut v)?;

        Ok(String::from_utf8(v)?)
    }

    #[cfg(feature = "async")]
    async fn parse_binary_async(
        r: &mut (impl futures::AsyncRead + Send + Unpin),
    ) -> anyhow::Result<Self> {
        let mut l = [0; std::mem::size_of::<u16>()];
        r.read_exact(&mut l).await?;
        let l = u16::from_be_bytes(l);

        let mut v = vec![0; l as usize];
        r.read_exact(&mut v).await?;

        Ok(String::from_utf8(v)?)
    }
}
#[cfg_attr(feature = "async", async_trait::async_trait)]
impl Writable for String {
    fn write_text(&self, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        write!(w, "{}{}", self, SEPARATOR)
    }

    fn write_binary(&self, w: &mut dyn std::io::Write) -> anyhow::Result<()> {
        w.write_all(&(self.len() as u16).to_be_bytes())?;
        w.write_all(self.as_bytes())?;
        Ok(())
    }

    #[cfg(feature = "async")]
    async fn write_binary_async(
        &self,
        w: &mut (dyn futures::AsyncWrite + Send + Unpin),
    ) -> anyhow::Result<()> {
        w.write_all(&(self.len() as u16).to_be_bytes()).await?;
        w.write_all(self.as_bytes()).await?;
        Ok(())
    }
}

#[cfg_attr(feature = "async", async_trait::async_trait)]
impl Readable for () {
    fn parse_text<'a>(_: &mut impl Iterator<Item = &'a str>) -> anyhow::Result<Self> {
        Ok(())
    }
    fn parse_binary(_: &mut impl std::io::Read) -> anyhow::Result<Self> {
        Ok(())
    }

    #[cfg(feature = "async")]
    async fn parse_binary_async(
        _: &mut (impl futures::AsyncRead + Send + Unpin),
    ) -> anyhow::Result<Self> {
        Ok(())
    }
}
#[cfg_attr(feature = "async", async_trait::async_trait)]
impl Writable for () {
    fn write_text(&self, _: &mut dyn std::fmt::Write) -> std::fmt::Result {
        Ok(())
    }

    fn write_binary(&self, _: &mut dyn std::io::Write) -> anyhow::Result<()> {
        Ok(())
    }

    #[cfg(feature = "async")]
    async fn write_binary_async(
        &self,
        _: &mut (dyn futures::AsyncWrite + Send + Unpin),
    ) -> anyhow::Result<()> {
        Ok(())
    }
}

#[cfg_attr(feature = "async", async_trait::async_trait)]
impl Readable for Duration {
    fn parse_text<'a>(s: &mut impl Iterator<Item = &'a str>) -> anyhow::Result<Self> {
        let Some(s) = s.next() else {
            return Err(anyhow::Error::msg("Not enough arguments"))
        };

        Ok(Duration::from_millis(s.parse::<u64>().map_err(|_| {
            anyhow::Error::msg("Couldn't parse millis")
        })?))
    }
    fn parse_binary(r: &mut impl std::io::Read) -> anyhow::Result<Self> {
        let mut millis = [0; std::mem::size_of::<u64>()];
        r.read_exact(&mut millis)?;
        Ok(Duration::from_millis(u64::from_be_bytes(millis)))
    }
    #[cfg(feature = "async")]
    async fn parse_binary_async(
        r: &mut (impl futures::AsyncRead + Send + Unpin),
    ) -> anyhow::Result<Self> {
        let mut millis = [0; std::mem::size_of::<u64>()];
        r.read_exact(&mut millis).await?;
        Ok(Duration::from_millis(u64::from_be_bytes(millis)))
    }
}
#[cfg_attr(feature = "async", async_trait::async_trait)]
impl Writable for Duration {
    fn write_text(&self, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        write!(w, "{}{}", self.as_millis() as u64, SEPARATOR)
    }
    fn write_binary(&self, w: &mut dyn std::io::Write) -> anyhow::Result<()> {
        w.write_all(&(self.as_millis() as u64).to_be_bytes())?;
        Ok(())
    }

    #[cfg(feature = "async")]
    async fn write_binary_async(
        &self,
        w: &mut (dyn futures::AsyncWrite + Send + Unpin),
    ) -> anyhow::Result<()> {
        w.write_all(&(self.as_millis() as u64).to_be_bytes())
            .await?;
        Ok(())
    }
}

#[cfg_attr(feature = "async", async_trait::async_trait)]
impl<T: Readable + Send> Readable for Option<T> {
    fn parse_text<'a>(s: &mut impl Iterator<Item = &'a str>) -> anyhow::Result<Self> {
        let Some(v) = s.next() else {
            return Err(anyhow::Error::msg("Not enough arguments"))
        };

        match v {
            "0" => Ok(None),
            "1" => Ok(Some(T::parse_text(s)?)),
            _ => Err(anyhow::Error::msg(
                "Option variant needs to be indicated with a 0 or a 1",
            )),
        }
    }

    fn parse_binary(r: &mut impl std::io::Read) -> anyhow::Result<Self> {
        let mut v = [0; std::mem::size_of::<u8>()];
        r.read_exact(&mut v)?;

        if u8::from_be_bytes(v) == 0 {
            return Ok(None);
        }

        Ok(Some(T::parse_binary(r)?))
    }
    #[cfg(feature = "async")]
    async fn parse_binary_async(
        r: &mut (impl futures::AsyncRead + Send + Unpin),
    ) -> anyhow::Result<Self> {
        let mut v = [0; std::mem::size_of::<u8>()];
        r.read_exact(&mut v).await?;

        if u8::from_be_bytes(v) == 0 {
            return Ok(None);
        }

        Ok(Some(T::parse_binary_async(r).await?))
    }
}
#[cfg_attr(feature = "async", async_trait::async_trait)]
impl<T: Writable + Sync> Writable for Option<T> {
    fn write_text(&self, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        if let Some(v) = self {
            write!(w, "1{}", SEPARATOR)?;
            v.write_text(w)
        } else {
            write!(w, "0{}", SEPARATOR)
        }
    }

    fn write_binary(&self, w: &mut dyn std::io::Write) -> anyhow::Result<()> {
        if let Some(v) = self {
            w.write_all(&[1])?;
            v.write_binary(w)?;
        } else {
            w.write_all(&[0])?;
        }
        Ok(())
    }

    #[cfg(feature = "async")]
    async fn write_binary_async(
        &self,
        w: &mut (dyn futures::AsyncWrite + Send + Unpin),
    ) -> anyhow::Result<()> {
        if let Some(v) = self {
            w.write_all(&[1]).await?;
            v.write_binary_async(w).await?;
        } else {
            w.write_all(&[0]).await?;
        }
        Ok(())
    }
}

#[cfg_attr(feature = "async", async_trait::async_trait)]
impl Readable for bool {
    fn parse_text<'a>(s: &mut impl Iterator<Item = &'a str>) -> anyhow::Result<Self> {
        let Some(s) = s.next() else {
            return Err(anyhow::Error::msg("Not enough arguments"))
        };

        match s {
            "0" => Ok(false),
            "1" => Ok(true),
            _ => Err(anyhow::Error::msg("Couldn't parse bool")),
        }
    }

    fn parse_binary(r: &mut impl std::io::Read) -> anyhow::Result<Self> {
        let mut b = [0];
        r.read_exact(&mut b)?;
        Ok(u8::from_be_bytes(b) != 0)
    }

    #[cfg(feature = "async")]
    async fn parse_binary_async(
        r: &mut (impl futures::AsyncRead + Send + Unpin),
    ) -> anyhow::Result<Self> {
        let mut b = [0];
        r.read_exact(&mut b).await?;
        Ok(u8::from_be_bytes(b) != 0)
    }
}
#[cfg_attr(feature = "async", async_trait::async_trait)]
impl Writable for bool {
    fn write_text(&self, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        write!(w, "{}{}", if *self { '1' } else { '0' }, SEPARATOR)
    }

    fn write_binary(&self, w: &mut dyn std::io::Write) -> anyhow::Result<()> {
        w.write_all(&(*self as u8).to_be_bytes())?;
        Ok(())
    }

    #[cfg(feature = "async")]
    async fn write_binary_async(
        &self,
        w: &mut (dyn futures::AsyncWrite + Send + Unpin),
    ) -> anyhow::Result<()> {
        w.write_all(&(*self as u8).to_be_bytes()).await?;
        Ok(())
    }
}

#[cfg_attr(feature = "async", async_trait::async_trait)]
impl Readable for char {
    fn parse_text<'a>(s: &mut impl Iterator<Item = &'a str>) -> anyhow::Result<Self> {
        let Some(s) = s.next() else {
            return Err(anyhow::Error::msg("Not enough arguments"))
        };

        s.chars()
            .next()
            .ok_or(anyhow::Error::msg("Couldn't get char"))
    }

    fn parse_binary(r: &mut impl std::io::Read) -> anyhow::Result<Self> {
        let mut b = [0; std::mem::size_of::<u32>()];
        r.read_exact(&mut b)?;
        char::from_u32(u32::from_be_bytes(b)).ok_or(anyhow::Error::msg("Couldn't parse char"))
    }

    #[cfg(feature = "async")]
    async fn parse_binary_async(
        r: &mut (impl futures::AsyncRead + Send + Unpin),
    ) -> anyhow::Result<Self> {
        let mut b = [0; std::mem::size_of::<u32>()];
        r.read_exact(&mut b).await?;
        char::from_u32(u32::from_be_bytes(b)).ok_or(anyhow::Error::msg("Couldn't parse char"))
    }
}
#[cfg_attr(feature = "async", async_trait::async_trait)]
impl Writable for char {
    fn write_text(&self, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        write!(w, "{}{}", self, SEPARATOR)
    }

    fn write_binary(&self, w: &mut dyn std::io::Write) -> anyhow::Result<()> {
        w.write_all(&(*self as u32).to_be_bytes())?;
        Ok(())
    }

    #[cfg(feature = "async")]
    async fn write_binary_async(
        &self,
        w: &mut (dyn futures::AsyncWrite + Send + Unpin),
    ) -> anyhow::Result<()> {
        w.write_all(&(*self as u32).to_be_bytes()).await?;
        Ok(())
    }
}

#[cfg_attr(feature = "async", async_trait::async_trait)]
impl<T: Readable + Send, const N: usize> Readable for [T; N] {
    fn parse_text<'a>(s: &mut impl Iterator<Item = &'a str>) -> anyhow::Result<Self> {
        let mut v = vec![];
        for _ in 0..N {
            v.push(T::parse_text(s)?);
        }

        // because no debug...
        Ok(unsafe { v.try_into().unwrap_unchecked() })
    }

    fn parse_binary(r: &mut impl std::io::Read) -> anyhow::Result<Self> {
        let mut v = vec![];
        for _ in 0..N {
            v.push(T::parse_binary(r)?);
        }

        // because no debug...
        Ok(unsafe { v.try_into().unwrap_unchecked() })
    }

    #[cfg(feature = "async")]
    async fn parse_binary_async(
        r: &mut (impl futures::AsyncRead + Send + Unpin),
    ) -> anyhow::Result<Self> {
        let mut v = vec![];
        for _ in 0..N {
            v.push(T::parse_binary_async(r).await?);
        }

        // because no debug...
        Ok(unsafe { v.try_into().unwrap_unchecked() })
    }
}
#[cfg_attr(feature = "async", async_trait::async_trait)]
impl<T: Writable + Sync, const N: usize> Writable for [T; N] {
    fn write_text(&self, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        for t in self {
            t.write_text(w)?;
        }
        Ok(())
    }
    fn write_binary(&self, w: &mut dyn std::io::Write) -> anyhow::Result<()> {
        for t in self {
            t.write_binary(w)?;
        }
        Ok(())
    }

    #[cfg(feature = "async")]
    async fn write_binary_async(
        &self,
        w: &mut (dyn futures::AsyncWrite + Send + Unpin),
    ) -> anyhow::Result<()> {
        for t in self {
            t.write_binary_async(w).await?;
        }
        Ok(())
    }
}

#[cfg_attr(feature = "async", async_trait::async_trait)]
impl Readable for IpAddr {
    fn parse_text<'a>(s: &mut impl Iterator<Item = &'a str>) -> anyhow::Result<Self> {
        match s.next() {
            Some("4") => {
                let Some(ip) = s.next() else {
                    return Err(anyhow::Error::msg("Not enough arguments"));
                };
                ip.parse()
                    .map_err(|_| anyhow::Error::msg("Couldn't parse ip"))
            }
            Some("6") => {
                let Some(ip) = s.next() else {
                    return Err(anyhow::Error::msg("Not enough arguments"));
                };
                ip.parse()
                    .map_err(|_| anyhow::Error::msg("Couldn't parse ip"))
            }

            Some(_) => Err(anyhow::Error::msg("Invalid ip type")),

            None => Err(anyhow::Error::msg("Not enough arguments")),
        }
    }
    fn parse_binary(r: &mut impl std::io::Read) -> anyhow::Result<Self> {
        let mut v = [0];
        r.read_exact(&mut v)?;
        match u8::from_be_bytes(v) {
            4 => {
                let mut ip = [0; 4];
                r.read_exact(&mut ip)?;
                Ok(Self::V4(std::net::Ipv4Addr::from(ip)))
            }
            6 => {
                let mut ip = [0; 16];
                r.read_exact(&mut ip)?;
                Ok(Self::V6(std::net::Ipv6Addr::from(ip)))
            }
            _ => Err(anyhow::Error::msg("Invalid ip type")),
        }
    }

    #[cfg(feature = "async")]
    async fn parse_binary_async(
        r: &mut (impl futures::AsyncRead + Send + Unpin),
    ) -> anyhow::Result<Self> {
        let mut v = [0];
        r.read_exact(&mut v).await?;
        match u8::from_be_bytes(v) {
            4 => {
                let mut ip = [0; 4];
                r.read_exact(&mut ip).await?;
                Ok(Self::V4(std::net::Ipv4Addr::from(ip)))
            }
            6 => {
                let mut ip = [0; 16];
                r.read_exact(&mut ip).await?;
                Ok(Self::V6(std::net::Ipv6Addr::from(ip)))
            }
            _ => Err(anyhow::Error::msg("Invalid ip type")),
        }
    }
}
#[cfg_attr(feature = "async", async_trait::async_trait)]
impl Writable for IpAddr {
    fn write_text(&self, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        match self {
            IpAddr::V4(ip) => write!(w, "4{}{}", SEPARATOR, ip),
            IpAddr::V6(ip) => write!(w, "6{}{}", SEPARATOR, ip),
        }
    }

    fn write_binary(&self, w: &mut dyn std::io::Write) -> anyhow::Result<()> {
        match self {
            IpAddr::V4(ip) => {
                w.write_all(&[4])?;
                w.write_all(&ip.octets())?;
            }
            IpAddr::V6(ip) => {
                w.write_all(&[6])?;
                w.write_all(&ip.octets())?;
            }
        }
        Ok(())
    }

    #[cfg(feature = "async")]
    async fn write_binary_async(
        &self,
        w: &mut (dyn futures::AsyncWrite + Send + Unpin),
    ) -> anyhow::Result<()> {
        match self {
            IpAddr::V4(ip) => {
                w.write_all(&[4]).await?;
                w.write_all(&ip.octets()).await?;
            }
            IpAddr::V6(ip) => {
                w.write_all(&[6]).await?;
                w.write_all(&ip.octets()).await?;
            }
        }
        Ok(())
    }
}

#[cfg_attr(feature = "async", async_trait::async_trait)]
impl Readable for SocketAddr {
    fn parse_text<'a>(s: &mut impl Iterator<Item = &'a str>) -> anyhow::Result<Self> {
        let ip: IpAddr = Readable::parse_text(s)?;
        let port: u16 = Readable::parse_text(s)?;
        Ok((ip, port).into())
    }
    fn parse_binary(r: &mut impl std::io::Read) -> anyhow::Result<Self> {
        let ip: IpAddr = Readable::parse_binary(r)?;
        let port: u16 = Readable::parse_binary(r)?;
        Ok((ip, port).into())
    }

    #[cfg(feature = "async")]
    async fn parse_binary_async(
        r: &mut (impl futures::AsyncRead + Send + Unpin),
    ) -> anyhow::Result<Self> {
        let ip: IpAddr = Readable::parse_binary_async(r).await?;
        let port: u16 = Readable::parse_binary_async(r).await?;
        Ok((ip, port).into())
    }
}
#[cfg_attr(feature = "async", async_trait::async_trait)]
impl Writable for SocketAddr {
    fn write_text(&self, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
        self.ip().write_text(w)?;
        self.port().write_text(w)
    }

    fn write_binary(&self, w: &mut dyn std::io::Write) -> anyhow::Result<()> {
        self.ip().write_binary(w)?;
        self.port().write_binary(w)?;
        Ok(())
    }

    #[cfg(feature = "async")]
    async fn write_binary_async(
        &self,
        w: &mut (dyn futures::AsyncWrite + Send + Unpin),
    ) -> anyhow::Result<()> {
        self.ip().write_binary_async(w).await?;
        self.port().write_binary_async(w).await?;
        Ok(())
    }
}

macro_rules! primitve_impl {
    ($t:tt) => {
        #[cfg_attr(feature = "async", async_trait::async_trait)]
        impl Readable for $t {
            fn parse_text<'a>(s: &mut impl Iterator<Item = &'a str>) -> anyhow::Result<Self> {
                let Some(s) = s.next() else {
                                            return Err(anyhow::Error::msg("Not enough arguments"))
                                        };

                s.parse()
                    .map_err(|_| anyhow::Error::msg("Couldn't parse primitive"))
            }
            fn parse_binary(r: &mut impl std::io::Read) -> anyhow::Result<Self> {
                let mut buf = [0; std::mem::size_of::<Self>()];
                r.read_exact(&mut buf)?;
                Ok(Self::from_be_bytes(buf))
            }

            #[cfg(feature = "async")]
            async fn parse_binary_async(
                r: &mut (impl futures::AsyncRead + Send + Unpin),
            ) -> anyhow::Result<Self> {
                let mut buf = [0; std::mem::size_of::<Self>()];
                r.read_exact(&mut buf).await?;
                Ok(Self::from_be_bytes(buf))
            }
        }
        #[cfg_attr(feature = "async", async_trait::async_trait)]
        impl Writable for $t {
            fn write_text(&self, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
                write!(w, "{}{}", self, SEPARATOR)
            }

            fn write_binary(&self, w: &mut dyn std::io::Write) -> anyhow::Result<()> {
                w.write_all(&self.to_be_bytes())?;
                Ok(())
            }

            #[cfg(feature = "async")]
            async fn write_binary_async(
                &self,
                w: &mut (dyn futures::AsyncWrite + Send + Unpin),
            ) -> anyhow::Result<()> {
                w.write_all(&self.to_be_bytes()).await?;
                Ok(())
            }
        }
    };
}

primitve_impl!(u8);
primitve_impl!(u16);
primitve_impl!(u32);
primitve_impl!(u64);
primitve_impl!(u128);

primitve_impl!(i8);
primitve_impl!(i16);
primitve_impl!(i32);
primitve_impl!(i64);
primitve_impl!(i128);

primitve_impl!(f32);
primitve_impl!(f64);

macro_rules! tuple_impl {
    ($($idx:tt $t:tt),+) => {
        #[cfg_attr(feature = "async", async_trait::async_trait)]
        impl<$($t,)+> Readable for ($($t,)+)
        where
            $($t: Readable + Send,)+
        {
            fn parse_text<'a>(s: &mut impl Iterator<Item = &'a str>) -> anyhow::Result<Self> {
                Ok(($(
                    $t :: parse_text(s)?,
                )+))
            }
            fn parse_binary(r: &mut impl std::io::Read) -> anyhow::Result<Self> {
                Ok(($(
                    $t :: parse_binary(r)?,
                )+))
            }
            #[cfg(feature = "async")]
            async fn parse_binary_async(
                r: &mut (impl futures::AsyncRead + Send + Unpin),
            ) -> anyhow::Result<Self> {
                Ok(($(
                    $t :: parse_binary_async(r).await?,
                )+))
            }
        }
        #[cfg_attr(feature = "async", async_trait::async_trait)]
        impl<$($t,)+> Writable for ($($t,)+)
        where
            $($t: Writable + Sync + Send,)+
        {
            fn write_text(&self, w: &mut dyn std::fmt::Write) -> std::fmt::Result {
                $(
                    self. $idx .write_text(w)?;
                )+

                Ok(())
            }

            fn write_binary(&self, w: &mut dyn std::io::Write) -> anyhow::Result<()> {
                $(
                    self. $idx .write_binary(w)?;
                )+

                Ok(())
            }

            #[cfg(feature = "async")]
            async fn write_binary_async(
                &self,
                w: &mut (dyn futures::AsyncWrite + Send + Unpin),
            ) -> anyhow::Result<()> {
                $(
                    self. $idx .write_binary_async(w).await?;
                )+

                Ok(())
            }
        }
    };
}

tuple_impl!(0 A, 1 B, 2 C, 3 D, 4 E, 5 F);
tuple_impl!(0 A, 1 B, 2 C, 3 D, 4 E);
tuple_impl!(0 A, 1 B, 2 C, 3 D);
tuple_impl!(0 A, 1 B, 2 C);
tuple_impl!(0 A, 1 B);
