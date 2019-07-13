use actix::*;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;

use bridge::server::Message;
use bridge::UserId;
use log::{info, warn};
use std::time::{Duration, Instant};
mod game;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

struct WsSession {
    id: UserId,
    hb: Instant,
    host: Addr<game::Host>,
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);

        let me = ctx.address();
        self.host
            .send(game::Connect {
                addr: me.recipient(),
            })
            .into_actor(self)
            .then(|result, actor, ctx| {
                match result {
                    Ok(result) => actor.id = result.id,
                    _ => ctx.stop(),
                }
                fut::ok(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        self.host.do_send(game::Disconnect { id: self.id });
        Running::Stop
    }
}

impl Handler<Message> for WsSession {
    type Result = ();

    fn handle(&mut self, message: Message, ctx: &mut Self::Context) {
        ctx.binary(serde_json::to_vec(&message).unwrap());
    }
}

impl StreamHandler<ws::Message, ws::ProtocolError> for WsSession {
    fn handle(&mut self, message: ws::Message, ctx: &mut Self::Context) {
        match message {
            ws::Message::Ping(ping) => {
                self.hb = Instant::now();
                ctx.pong(&ping);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Binary(binary) => {
                use bridge::client::Message;
                if let Ok(message) = serde_json::from_slice::<Message>(&binary) {
                    match message {
                        Message::Chat { text, to } => {
                            self.host.do_send(game::Chat {
                                id: self.id,
                                text,
                                to,
                            });
                        }
                        Message::CreateRoom { room } => {
                            self.host.do_send(game::CreateRoom {
                                id: self.id,
                                room,
                            });
                        }
                        Message::GetRoomDetail { room } => {
                            self.host.do_send(game::GetRoomDetail {
                                id: self.id,
                                room,
                            });
                        }
                        Message::JoinRoom { room } => {
                            self.host.do_send(game::JoinRoom {
                                id: self.id,
                                room,
                            });
                        }
                        Message::QuitRoom => {
                            self.host.do_send(game::QuitRoom {
                                id: self.id,
                            });
                        }
                    }
                }
            }
            ws::Message::Close(_) => {
                ctx.stop();
            }
            ws::Message::Text(_) => warn!("Unexpected Text"),
            ws::Message::Nop => (),
        }
    }
}

impl WsSession {
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |actor, ctx| {
            if Instant::now().duration_since(actor.hb) > CLIENT_TIMEOUT {
                info!("Client {} heartbeat failed.", actor.id);

                actor.host.do_send(game::Disconnect { id: actor.id });

                ctx.stop();
                return;
            }
            ctx.ping("");
        });
    }
}

fn game_route(
    req: HttpRequest,
    stream: web::Payload,
    server: web::Data<Addr<game::Host>>,
) -> Result<HttpResponse, Error> {
    ws::start(
        WsSession {
            id: 0,
            hb: Instant::now(),
            host: server.get_ref().clone(),
        },
        &req,
        stream,
    )
}

fn main() -> std::io::Result<()> {
    simple_logger::init_with_level(log::Level::Info).unwrap();
    let sys = System::new("kiwitu");
    let server = game::Host::default().start();
    HttpServer::new(move || {
        App::new()
            .data(server.clone())
            .service(web::resource("/ws/").to(game_route))
            .service(actix_files::Files::new("/", "static/").index_file("index.html"))
    })
    .bind("127.0.0.1:8080")?
    .start();

    sys.run()
}
