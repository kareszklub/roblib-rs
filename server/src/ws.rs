use actix::{Actor, ActorContext, AsyncContext, StreamHandler};
use actix_web_actors::ws::{self as ws_actix, Message, WebsocketContext};
use futures_util::stream::once;
use roblib::{cmd::Cmd, Robot};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct WebSocket {
    hb: Instant,
    roland: Arc<Robot>,
}

impl Actor for WebSocket {
    type Context = WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
    }
}

impl StreamHandler<Result<ws_actix::Message, ws_actix::ProtocolError>> for WebSocket {
    fn handle(
        &mut self,
        msg: Result<ws_actix::Message, ws_actix::ProtocolError>,
        ctx: &mut Self::Context,
    ) {
        debug!("WS: {:?}", msg);
        match msg {
            Ok(Message::Text(text)) => {
                let robot_pointer = self.roland.clone();
                ctx.add_stream(once(async move {
                    Ok(Message::Text(
                        (Cmd::exec_str(&text, robot_pointer.as_ref()).await).into(),
                    ))
                }));
            }

            Ok(Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }

            Ok(Message::Pong(_)) => self.hb = Instant::now(),

            Ok(Message::Binary(_)) => ctx.text("binary data not supported"),

            Ok(Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }

            _ => ctx.stop(),
        }
    }
}

impl WebSocket {
    pub fn new(roland: Arc<Robot>) -> Self {
        Self {
            hb: Instant::now(),
            roland,
        }
    }

    /// helper method that sends pings to the client.
    fn hb(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
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
