use actix_codec::Framed;
use actix_http::ws::{CloseReason, Frame, ProtocolError};
use actix_web::web::Bytes;
use anyhow::{anyhow, Result};
use awc::{
    error::WsClientError,
    ws::{Codec, Message},
    BoxedSocket, Client,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use roblib_shared::cmd::{self, get_time};
use std::fmt::Debug;

type WsConnection = Framed<BoxedSocket, Codec>;

#[derive(Debug)]
pub enum RobotError {
    WsClientError(WsClientError),
    ProtocolError(ProtocolError),
    UnsupportedFrameType(String),
    SocketClosed(Option<CloseReason>),
    InvalidResponse(String),
}

pub struct Robot {
    ws: WsConnection,
}

impl Robot {
    fn new(ws: WsConnection) -> Self {
        Self { ws }
    }

    pub async fn connect(addr: &str) -> Result<Self> {
        let (_, ws) = match Client::new().ws(addr).connect().await {
            Ok(x) => x,
            Err(_) => return Err(anyhow!("failed to connect")),
        };
        Ok(Self::new(ws))
    }

    async fn send(&mut self, cmd: &str) -> Result<String> {
        self.ws.send(Message::Text(cmd.to_string())).await?;

        while let Some(Ok(msg)) = self.ws.next().await {
            match msg {
                Frame::Text(b) => return Ok(String::from_utf8(b.to_vec()).unwrap()),
                Frame::Binary(_) => return Err(anyhow!("received binary frame")),
                Frame::Continuation(_) => return Err(anyhow!("received continuation frame")),
                Frame::Ping(_) => {
                    self.ws.send(Message::Pong(Bytes::new())).await?;
                    trace!("ping");
                    continue;
                }
                Frame::Pong(_) => {
                    trace!("pong");
                    continue;
                }
                Frame::Close(reason) => {
                    self.ws.close().await?;
                    return Err(anyhow!("socket closed: {reason:?}"));
                }
            }
        }

        unreachable!("should have received a message");
    }

    pub async fn disconnect(&mut self) -> Result<()> {
        info!("disconnecting");
        Ok(self.ws.close().await?)
    }

    pub async fn move_robot(&mut self, left: i8, right: i8) -> Result<String> {
        let s = format!("m {} {}", left, right);
        debug!("S: {}", s);
        self.send(&s).await
    }
    pub async fn stop_robot(&mut self) -> Result<String> {
        debug!("S: s");
        self.send("s").await
    }
    pub async fn led(&mut self, (r, g, b): (bool, bool, bool)) -> Result<String> {
        let s = format!("l {} {} {}", r as i8, g as i8, b as i8);
        debug!("S: {}", s);
        self.send(&s).await
    }
    pub async fn servo_absolute(&mut self, angle: f32) -> Result<String> {
        let s = format!("v {}", angle);
        debug!("S: {}", s);
        self.send(&s).await
    }
    pub async fn buzzer(&mut self, freq: u16) -> Result<String> {
        let s = format!("b {}", freq);
        debug!("S: {}", s);
        self.send(&s).await
    }
    pub async fn get_sensor_data(&mut self) -> Result<cmd::SensorData> {
        debug!("S: t");
        let d = self
            .send("t")
            .await?
            .split(',')
            .map(|s| s.parse::<i32>().unwrap())
            .collect::<Vec<_>>()
            .try_into()
            .unwrap_or_else(|v: Vec<_>| {
                panic!("Expected a Vec of length {} but it was {}", 4, v.len())
            });
        debug!("R {:?}", d);
        Ok(d)
    }
    pub async fn measure_latency(&mut self) -> Result<f64> {
        let now = get_time();
        let s = format!("z {}", now);
        debug!("S: {}", s);
        let r = self.send(&s).await?;
        debug!("R {}", &r);
        match r.parse() {
            Ok(x) => Ok(x),
            Err(_) => Err(anyhow!("invalid response: {r:?}")),
        }
    }
}
