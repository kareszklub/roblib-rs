use actix::{prelude::*, Actor, ActorContext, AsyncContext, Handler, StreamHandler};
use actix_web::{
    get,
    web::{Bytes, Data, Payload},
    HttpRequest, HttpResponse,
};
use actix_web_actors::ws::{Message, ProtocolError, WebsocketContext};
use roblib::text_format;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use crate::cmd::execute_concrete;
use crate::Backends;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// websocket endpoint
/// will attempt to upgrade to websocket connection
#[get("/ws")]
pub(crate) async fn index(
    req: HttpRequest,
    stream: Payload,
    robot: Data<Arc<Backends>>,
) -> Result<HttpResponse, actix_web::Error> {
    actix_web_actors::ws::start(WebSocket::new(robot.as_ref().clone()), &req, stream)
}

pub(crate) struct WebSocket {
    robot: Arc<Backends>,
    last_heartbeat: Instant,
}

enum BinaryOrText {
    Text(String),
    Binary(Bytes),
}

#[derive(Message)]
#[rtype(result = "()")]
struct CmdResult(anyhow::Result<Option<BinaryOrText>>);

impl Handler<CmdResult> for WebSocket {
    type Result = ();

    fn handle(&mut self, res: CmdResult, ctx: &mut Self::Context) {
        match res.0 {
            Ok(Some(ret)) => match ret {
                BinaryOrText::Text(ret) => ctx.text(ret),
                BinaryOrText::Binary(b) => ctx.binary(b),
            },
            Ok(None) => (),
            Err(e) => {
                let e = e.to_string();
                error!("{e}");
                ctx.text(e);
            }
        }
    }
}

impl Actor for WebSocket {
    type Context = WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.set_mailbox_capacity(128);
        self.start_heartbeat(ctx);
    }
}

impl StreamHandler<Result<Message, ProtocolError>> for WebSocket {
    fn handle(&mut self, msg: Result<Message, ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Ok(msg) => msg,
            Err(e) => {
                error!("{e}");
                ctx.stop();
                return;
            }
        };

        match msg {
            Message::Text(text) => {
                let recipient = ctx.address().recipient();
                let robot_pointer = self.robot.clone();

                // FIXME: commands get executed all at once, on disconnect
                async move {
                    let res = match roblib::text_format::de::from_str(&text) {
                        Ok(c) => {
                            let mut s = String::new();
                            execute_concrete(
                                c,
                                robot_pointer,
                                &mut text_format::ser::Serializer::new(&mut s),
                            )
                            .await
                            .map(move |_| s)
                        }
                        Err(e) => Err(e.into()),
                    };

                    let res = match res {
                        Ok(s) if s.is_empty() => Ok(None),
                        Ok(s) => Ok(Some(BinaryOrText::Text(s))),
                        Err(e) => Err(e),
                    };

                    recipient.do_send(CmdResult(res))
                }
                .into_actor(self)
                .spawn(ctx);
            }

            Message::Binary(b) => {
                let recipient = ctx.address().recipient();
                let robot_pointer = self.robot.clone();

                // FIXME: commands get executed all at once, on disconnect
                async move {
                    let res = match postcard::from_bytes(&b) {
                        Ok(c) => {
                            let mut serializer = postcard::Serializer {
                                output: postcard::ser_flavors::StdVec::new(),
                            };

                            match execute_concrete(c, robot_pointer, &mut serializer).await {
                                Ok(r) => {
                                    if let Some(()) = r {
                                        postcard::ser_flavors::Flavor::finalize(serializer.output)
                                            .map(|b| BinaryOrText::Binary(Bytes::from(b)))
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

                    recipient.do_send(CmdResult(res))
                }
                .into_actor(self)
                .spawn(ctx);
            }

            Message::Ping(msg) => {
                self.last_heartbeat = Instant::now();
                ctx.pong(&msg);
            }

            Message::Pong(_) => self.last_heartbeat = Instant::now(),

            Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }

            m => error!("got unsupported message {m:?}"),
        }
    }
}

impl WebSocket {
    pub fn new(robot: Arc<Backends>) -> Self {
        Self {
            last_heartbeat: Instant::now(),
            robot,
        }
    }

    fn start_heartbeat(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.last_heartbeat) > CLIENT_TIMEOUT {
                // heartbeat timed out
                debug!("Websocket Client heartbeat failed, disconnecting!");
                // stop actor
                ctx.stop();
                return;
            }

            ctx.ping(b"");
        });
    }
}
