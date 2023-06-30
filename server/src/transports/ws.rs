use actix::{prelude::*, Actor, ActorContext, AsyncContext, Handler, StreamHandler};
use actix_web::{
    get,
    web::{Data, Payload},
    HttpRequest, HttpResponse,
};
use actix_web_actors::ws::{Message, ProtocolError, WebsocketContext};
use roblib::cmd::Concrete;
use roblib_parsing::{Readable, Writable, SEPARATOR};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use crate::cmd::execute_concrete;
use crate::{AppState, Robot};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// websocket endpoint
/// will attempt to upgrade to websocket connection
#[get("/ws")]
pub(crate) async fn index(
    req: HttpRequest,
    stream: Payload,
    state: Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    actix_web_actors::ws::start(WebSocket::new(state.robot.clone()), &req, stream)
}

pub(crate) struct WebSocket {
    robot: Arc<Robot>,
    last_heartbeat: Instant,
}

#[derive(Message)]
#[rtype(result = "()")]
struct CmdResult(anyhow::Result<Option<Box<dyn Writable + Send + Sync>>>);

impl Handler<CmdResult> for WebSocket {
    type Result = ();

    fn handle(&mut self, res: CmdResult, ctx: &mut Self::Context) {
        match res.0 {
            Ok(Some(ret)) => {
                let mut s = String::new();
                match ret.write_text(&mut s) {
                    Ok(()) => ctx.text(&*s),
                    Err(e) => ctx.text(e.to_string()),
                }
            }
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
                    let res = match Concrete::parse_text(&mut text.split(SEPARATOR)) {
                        Ok(c) => execute_concrete(c, robot_pointer).await,
                        Err(e) => Err(e),
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

            Message::Binary(_) => ctx.text("binary data not supported"),

            Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }

            m => error!("got unsupported message {m:?}"),
        }
    }
}

impl WebSocket {
    pub fn new(robot: Arc<Robot>) -> Self {
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
