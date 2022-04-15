#[macro_use]
extern crate log;

use actix_codec::Framed;
use actix_http::ws::{CloseReason, Frame, ProtocolError};
use actix_web::web::Bytes;
use awc::{
    error::WsClientError,
    ws::{Codec, Message},
    BoxedSocket, Client,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use roblib_shared::cmd::get_time;
use std::convert::TryInto;
use std::{fmt::Debug, result::Result as StdResult};

pub use roblib_shared::{cmd, logger};

type WsConnection = Framed<BoxedSocket, Codec>;

pub type Result = StdResult<(), RobotError>;
pub enum RobotError {
    WsClientError(WsClientError),
    ProtocolError(ProtocolError),
    UnsupportedFrameType(String),
    SocketClosed(Option<CloseReason>),
    InvalidResponse(String),
}
impl From<WsClientError> for RobotError {
    fn from(err: WsClientError) -> Self {
        RobotError::WsClientError(err)
    }
}
impl From<ProtocolError> for RobotError {
    fn from(err: ProtocolError) -> Self {
        RobotError::ProtocolError(err)
    }
}
impl Debug for RobotError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RobotError::WsClientError(err) => write!(f, "WsClientError: {}", err),
            RobotError::ProtocolError(err) => write!(f, "ProtocolError: {}", err),
            RobotError::UnsupportedFrameType(fr) => write!(f, "UnsupportedFrameType: {}", fr),
            RobotError::SocketClosed(cr) => write!(f, "SocketClosed: {:?}", cr),
            RobotError::InvalidResponse(r) => write!(f, "InvalidResponse: {}", r),
        }
    }
}

pub struct Robot {
    ws: WsConnection,
}

impl Robot {
    fn new(ws: WsConnection) -> Robot {
        Robot { ws }
    }

    pub async fn connect(addr: &str) -> StdResult<Robot, RobotError> {
        let (_, ws) = Client::new().ws(addr).connect().await?;
        Ok(Robot::new(ws))
    }

    async fn send(&mut self, cmd: &str) -> StdResult<String, RobotError> {
        self.ws.send(Message::Text(cmd.to_string())).await?;

        while let Some(Ok(msg)) = self.ws.next().await {
            match msg {
                Frame::Text(b) => return Ok(String::from_utf8(b.to_vec()).unwrap()),
                Frame::Binary(_) => {
                    return Err(RobotError::UnsupportedFrameType("binary".to_string()))
                }
                Frame::Continuation(_) => {
                    return Err(RobotError::UnsupportedFrameType("continuation".to_string()))
                }
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
                    return Err(RobotError::SocketClosed(reason));
                }
            }
        }

        unreachable!("should have received a message");
    }

    pub async fn disconnect(&mut self) -> StdResult<(), RobotError> {
        info!("disconnecting");
        Ok(self.ws.close().await?)
    }

    pub async fn move_robot(&mut self, left: i8, right: i8) -> StdResult<String, RobotError> {
        let s = format!("m {} {}", left, right);
        debug!("S: {}", s);
        self.send(&s).await
    }
    pub async fn stop_robot(&mut self) -> StdResult<String, RobotError> {
        debug!("S: s");
        self.send("s").await
    }
    pub async fn led(&mut self, (r, g, b): (bool, bool, bool)) -> StdResult<String, RobotError> {
        let s = format!("l {} {} {}", r as i8, g as i8, b as i8);
        debug!("S: {}", s);
        self.send(&s).await
    }
    pub async fn servo_absolute(&mut self, angle: f32) -> StdResult<String, RobotError> {
        let s = format!("v {}", angle);
        debug!("S: {}", s);
        self.send(&s).await
    }
    pub async fn buzzer(&mut self, freq: u16) -> StdResult<String, RobotError> {
        let s = format!("b {}", freq);
        debug!("S: {}", s);
        self.send(&s).await
    }
    pub async fn get_sensor_data(&mut self) -> StdResult<cmd::SensorData, RobotError> {
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
    pub async fn measure_latency(&mut self) -> StdResult<f64, RobotError> {
        let now = get_time();
        let s = format!("z {}", now);
        debug!("S: {}", s);
        let r = self.send(&s).await?;
        debug!("R {}", &r);
        match r.parse() {
            Ok(x) => Ok(x),
            Err(_) => Err(RobotError::InvalidResponse(r)),
        }
    }
}
