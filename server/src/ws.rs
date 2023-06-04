use actix::prelude::*;
use actix::{Actor, ActorContext, AsyncContext, Handler, StreamHandler};
use actix_web_actors::ws::{Message, ProtocolError, WebsocketContext};
use roblib::{cmd::Cmd, Robot};
use std::{
    str::FromStr,
    sync::Arc,
    time::{Duration, Instant},
};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct WebSocket {
    last_heartbeat: Instant,
    roland: Arc<Robot>,
}

#[derive(Debug, Message)]
#[rtype(result = "()")]
struct CmdResult(anyhow::Result<Option<String>>);

impl Handler<CmdResult> for WebSocket {
    type Result = ();
    fn handle(&mut self, CmdResult(msg): CmdResult, ctx: &mut Self::Context) {
        match msg {
            Ok(Some(res)) => ctx.text(res),
            Ok(None) => ctx.text(""),
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

        debug!("WS: {msg:?}");
        match msg {
            Message::Text(text) => {
                let cmd = Cmd::from_str(&text).unwrap();
                let robot_pointer = self.roland.clone();

                let recipient = ctx.address().recipient();
                async move { recipient.do_send(CmdResult(cmd.exec(&robot_pointer).await)) }
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
    pub fn new(roland: Arc<Robot>) -> Self {
        Self {
            last_heartbeat: Instant::now(),
            roland,
        }
    }

    /// helper method that sends pings to the client.
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
