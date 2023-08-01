use crate::{
    cmd::execute_concrete, event_bus::sub::SubStatus, transports::SubscriptionId, Backends,
};
use axum::{
    extract::{
        ws::{Message, WebSocket},
        ConnectInfo, State, WebSocketUpgrade,
    },
    response::IntoResponse,
};
use roblib::{cmd, event::ConcreteValue, text_format};
use std::{fmt::Write, io::Cursor, net::SocketAddr, sync::Arc};
use tokio::sync::broadcast::{Receiver, Sender};

pub type Id = SocketAddr;
pub type SubId = u32;
pub type Item = (Id, SubId);
pub type Tx = Sender<(ConcreteValue, Item)>;
pub type Rx = Receiver<(ConcreteValue, Item)>;

pub(crate) async fn ws_route(
    State((robot, rx)): super::http::AppState,
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        if let Err(e) = ws_handler(socket, addr, robot, rx.resubscribe()).await {
            log::error!("ws error: {e}");
        }
    })
}

enum Action {
    ClientMessage(Message),
    Event(ConcreteValue, Item),
    Disconnect,
    ServerAbort,
}

async fn ws_handler(
    mut socket: WebSocket,
    addr: SocketAddr,
    robot: Arc<Backends>,
    mut rx: Rx,
) -> anyhow::Result<()> {
    let bin = bincode::options();

    loop {
        let action = tokio::select! {
            _ = robot.abort_token.cancelled() => Action::ServerAbort,
            res = socket.recv() => match res {
                Some(msg) => Action::ClientMessage(msg?),
                None => Action::Disconnect,
            },
            Ok(msg) = rx.recv() => Action::Event(msg.0, msg.1),
        };
        match action {
            Action::ClientMessage(msg) => {
                let (id, cmd): (u32, cmd::Concrete) = match &msg {
                    Message::Text(s) => match text_format::de::from_str(s) {
                        Ok(d) => d,
                        Err(e) => {
                            log::error!("text_format error: {e}");
                            continue;
                        }
                    },
                    Message::Binary(b) => match bincode::Options::deserialize(bin, b) {
                        Ok(d) => d,
                        Err(e) => {
                            log::error!("bincode error: {e}");
                            continue;
                        }
                    },
                    Message::Close(close) => {
                        if let Some(close) = close {
                            log::debug!("ws close, reason: {}", close.reason);
                        }
                        continue;
                    }
                    _ => continue,
                };

                match cmd {
                    cmd::Concrete::Subscribe(c) => {
                        let sub = SubscriptionId::Ws(addr, id);
                        dbg!(&sub);
                        if let Err(e) = robot.sub.send((c.0, sub, SubStatus::Subscribe)) {
                            log::error!("event bus sub error: {e}");
                        };
                    }
                    cmd::Concrete::Unsubscribe(c) => {
                        let unsub = SubscriptionId::Ws(addr, id);
                        dbg!(&unsub);
                        if let Err(e) = robot.sub.send((c.0, unsub, SubStatus::Unsubscribe)) {
                            log::error!("event bus sub error: {e}");
                        };
                    }

                    _ => match &msg {
                        Message::Text(_) => {
                            let mut buf = String::new();
                            let mut ser = text_format::ser::Serializer::new(&mut buf);
                            write!(ser, "{id}")?;
                            let res = execute_concrete(cmd, robot.clone(), &mut ser).await?;
                            if res.is_some() {
                                dbg!(&buf);
                                socket.send(Message::Text(buf)).await?;
                            }
                        }
                        Message::Binary(_) => {
                            let mut v = Vec::new();
                            let mut c = Cursor::new(&mut v);
                            bincode::Options::serialize_into(bin, &mut c, &id)?;
                            let res = execute_concrete(
                                cmd,
                                robot.clone(),
                                &mut bincode::Serializer::new(&mut c, bin),
                            )
                            .await?;
                            dbg!(&res);
                            if res.is_some() {
                                dbg!();
                                socket.send(Message::Binary(v)).await?;
                            }
                        }
                        _ => unreachable!(),
                    },
                }
            }
            Action::Event(ev, (ev_addr, id)) => {
                if addr != ev_addr {
                    continue;
                }
                let data: Vec<u8> = match ev {
                    #[cfg(feature = "roland")]
                    ConcreteValue::TrackSensor(v) => bincode::Options::serialize(bin, &(id, v))?,
                    #[cfg(feature = "roland")]
                    ConcreteValue::UltraSensor(v) => bincode::Options::serialize(bin, &(id, v))?,
                    #[cfg(feature = "gpio")]
                    ConcreteValue::GpioPin(v) => bincode::Options::serialize(bin, &(id, v))?,
                    #[cfg(feature = "camloc")]
                    ConcreteValue::CamlocConnect(v) => bincode::Options::serialize(bin, &(id, v))?,
                    #[cfg(feature = "camloc")]
                    ConcreteValue::CamlocDisconnect(v) => {
                        bincode::Options::serialize(bin, &(id, v))?
                    }
                    #[cfg(feature = "camloc")]
                    ConcreteValue::CamlocPosition(v) => bincode::Options::serialize(bin, &(id, v))?,
                    #[cfg(feature = "camloc")]
                    ConcreteValue::CamlocInfoUpdate(v) => {
                        bincode::Options::serialize(bin, &(id, v))?
                    }
                    ConcreteValue::None => continue,
                };
                socket.send(Message::Binary(data)).await?;
            }

            Action::Disconnect => {
                log::debug!("ws client disconnected: {addr}");
                return Ok(());
            }
            Action::ServerAbort => {
                log::debug!("abort: ws {addr}");
                return Ok(());
            }
        }
    }
}
