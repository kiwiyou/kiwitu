use actix::prelude::*;
use bridge::server::Message;
use bridge::*;
use log::info;
use rand::{rngs::ThreadRng, Rng};
use std::collections::HashMap;

#[derive(Message)]
#[rtype(Welcome)]
pub struct Connect {
    pub addr: Recipient<Message>,
}

#[derive(MessageResponse)]
pub struct Welcome {
    pub id: UserId,
}

#[derive(Message)]
pub struct Disconnect {
    pub id: UserId,
}

#[derive(Message)]
pub struct Chat {
    pub id: UserId,
    pub text: String,
    pub to: Option<UserId>,
}

#[derive(Message)]
pub struct CreateRoom {
    pub id: UserId,
    pub room: RoomBrief,
}

#[derive(Message)]
pub struct GetRoomDetail {
    pub id: UserId,
    pub room: RoomId,
}

#[derive(Message)]
pub struct JoinRoom {
    pub id: UserId,
    pub room: RoomId,
}

#[derive(Message)]
pub struct QuitRoom {
    pub id: UserId,
}

#[derive(Clone)]
struct Session {
    id: UserId,
    pipe: Recipient<Message>,
    room: Option<RoomId>,
    name: String,
}

pub struct Host {
    sessions: HashMap<UserId, Session>,
    rng: ThreadRng,
    rooms: HashMap<RoomId, Room>,
}

pub struct Room {
    id: RoomId,
    title: String,
    members: Vec<Session>,
    owner: UserId,
}

impl Host {
    fn generate_guest(&mut self) -> (UserId, String) {
        const NAME_LEN: u32 = 4;
        let name_char = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz"
            .chars()
            .collect::<Vec<char>>();
        let id = self.rng.gen_range(0, name_char.len().pow(NAME_LEN) - 1);
        let mut name = String::from("GUEST_");
        let mut id_copy = id;
        for _ in 0..NAME_LEN {
            name.push(name_char[id_copy % name_char.len()]);
            id_copy /= name_char.len();
        }
        (id, name)
    }
}

impl Default for Host {
    fn default() -> Self {
        Self {
            sessions: HashMap::new(),
            rng: rand::thread_rng(),
            rooms: HashMap::new(),
        }
    }
}

impl Actor for Host {
    type Context = Context<Self>;
}

impl Handler<Connect> for Host {
    type Result = Welcome;

    fn handle(&mut self, message: Connect, _: &mut Context<Self>) -> Self::Result {
        let (id, name) = self.generate_guest();
        info!("User {} joined", name);
        let _ = message.addr.do_send(Message::Welcome {
            id,
            users: self
                .sessions
                .values()
                .map(|session| UserBrief {
                    id: session.id,
                    name: session.name.clone(),
                })
                .collect(),
            rooms: self
                .rooms
                .values()
                .map(|room| RoomBrief {
                    id: room.id,
                    title: room.title.clone(),
                })
                .collect(),
        });
        self.sessions.insert(
            id,
            Session {
                id,
                name: name.clone(),
                pipe: message.addr,
                room: None,
            },
        );
        for (_, session) in self.sessions.iter() {
            let _ = session.pipe.do_send(Message::Connected {
                user: UserBrief {
                    id,
                    name: name.clone(),
                },
            });
        }
        Welcome { id }
    }
}

impl Handler<Disconnect> for Host {
    type Result = ();

    fn handle(&mut self, message: Disconnect, ctx: &mut Context<Self>) {
        let id = message.id;
        self.handle(QuitRoom {
            id
        }, ctx);
        if let Some(session) = self.sessions.get(&id) {
            info!("User {} quit", session.name);
            self.sessions.remove(&id);
            for (_, session) in self.sessions.iter() {
                let _ = session.pipe.do_send(Message::Disconnected { id });
            }
        }
    }
}

impl Handler<Chat> for Host {
    type Result = ();

    fn handle(&mut self, message: Chat, _: &mut Context<Self>) {
        let text = message.text.trim().to_string();
        if text.len() < 1 {
            return;
        }
        let from = message.id;
        if let Some(to) = message.to {
            if let Some(session) = self.sessions.get(&to) {
                let _ = session.pipe.do_send(Message::Chat {
                    from,
                    text,
                    whisper: true,
                });
            } else if let Some(from_session) = self.sessions.get(&from) {
                let _ = from_session
                    .pipe
                    .do_send(Message::Alert(Alert::TargetNotFound));
            }
        } else {
            let iter: Box<dyn Iterator<Item = &Session>> = if let Some(room) = self
                .sessions
                .get(&from)
                .and_then(|session| session.room)
                .and_then(|room| self.rooms.get(&room))
            {
                Box::new(room.members.iter())
            } else {
                Box::new(self.sessions.values())
            };
            for session in iter {
                let _ = session.pipe.do_send(Message::Chat {
                    from,
                    text: text.clone(),
                    whisper: false,
                });
            }
        }
    }
}

const ROOM_LIMIT: RoomId = 1000;

impl Handler<CreateRoom> for Host {
    type Result = ();

    fn handle(&mut self, message: CreateRoom, _: &mut Context<Self>) {
        if let Some(mut session) = self.sessions.get_mut(&message.id) {
            // Creating a room in a room?!
            if session.room.is_some() {
                return;
            }
            let title = message.room.title.trim().to_string();
            if title.len() < 1 {
                return;
            }
            let room_id = self.rng.gen_range(0, ROOM_LIMIT);
            let room = Room {
                id: room_id,
                members: vec![session.clone()],
                owner: session.id,
                title: title.clone(),
            };
            session.room = Some(room_id);
            let _ = session.pipe.do_send(Message::ReadyJoin {
                room: (&room).into(),
            });
            info!("User {} created room #{}", session.name, room_id);
            self.rooms.insert(room_id, room);
            for session in self.sessions.values() {
                let _ = session.pipe.do_send(Message::NewRoom {
                    room: RoomBrief {
                        id: room_id,
                        title: title.clone(),
                    }
                });
            }
        }
    }
}

impl Handler<GetRoomDetail> for Host {
    type Result = ();

    fn handle(&mut self, message: GetRoomDetail, _: &mut Context<Self>) {
        if let Some(session) = self.sessions.get(&message.id) {
            if let Some(room) = self.rooms.get(&message.room) {
                let _ = session.pipe.do_send(Message::RoomDetail {
                    room: room.into(),
                });
            }
        }
    }
}

impl Handler<JoinRoom> for Host {
    type Result = ();

    fn handle(&mut self, message: JoinRoom, _: &mut Context<Self>) {
        if let Some(session) = self.sessions.get_mut(&message.id) {
            if session.room.is_some() {
                return;
            }
            if let Some(room) = self.rooms.get_mut(&message.room) {
                session.room = Some(room.id);
                room.members.push(session.clone());
                let _ = session.pipe.do_send(Message::ReadyJoin {
                    room: (&*room).into(),
                });
                for session in room.members.iter() {
                    let _ = session.pipe.do_send(Message::Alert(Alert::Join { user: message.id }));
                    let _ = session.pipe.do_send(Message::RoomUpdate { room: (&*room).into() });
                }
            }
        }
    }
}

impl Handler<QuitRoom> for Host {
    type Result = ();

    fn handle(&mut self, message: QuitRoom, _: &mut Context<Self>) {
        if let Some(session) = self.sessions.get_mut(&message.id) {
            let reflex = session.clone();
            session.room = None;
            let mut to_destroy = None;
            if let Some(room) = reflex.room.and_then(|room_id| self.rooms.get_mut(&room_id)) {
                room.members.retain(|member| member.id != reflex.id);
                if room.members.len() < 1 {
                    to_destroy = Some(room.id);
                } else if room.owner == reflex.id {
                    room.owner = room.members[0].id;
                }
                for session in room.members.iter() {
                    let _ = session.pipe.do_send(Message::Alert(Alert::Quit { user: reflex.id }));
                    let _ = session.pipe.do_send(Message::RoomUpdate { room: (&*room).into() });
                }
            }
            if let Some(room_id) = to_destroy {
                self.rooms.remove(&room_id);
                for session in self.sessions.values() {
                    let _ = session.pipe.do_send(Message::DestroyRoom {
                        room: room_id,
                    });
                }
            }
        }
    }
}

impl From<&Room> for bridge::Room {
    fn from(room: &Room) -> Self {
        Self {
            id: room.id,
            title: room.title.clone(),
            owner: room.owner,
            members: room.members.iter().map(|session| session.id).collect(),
        }
    }
}