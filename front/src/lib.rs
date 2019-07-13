#![recursion_limit = "256"]
use bridge::{Alert, UserBrief, UserId, RoomBrief, RoomId, Room};
use failure::Error;
use std::collections::HashMap;
use yew::format::Json;
use yew::services::websocket::{WebSocketService, WebSocketStatus, WebSocketTask};
use yew::{html, Component, ComponentLink, Html, Renderable, ShouldRender};

mod module;
use module::*;

pub struct Model {
    ws: Option<WebSocketTask>,
    link: ComponentLink<Self>,
    socket: WebSocketService,
    connected: Option<bool>,
    client: Option<Client>,

    menu: Option<MenuItem>,
}

struct Client {
    id: UserId,
    room: Option<Room>,
    users: HashMap<UserId, UserBrief>,
    rooms: HashMap<RoomId, RoomBrief>,
    chats: Vec<Chat>,
}

pub enum Msg {
    WebResponse(Result<bridge::server::Message, Error>),
    WebRequest(bridge::client::Message),
    Connect,
    Connected,
    Failed,
    SendChat(String),
    OpenMenu(MenuItem),
    MenuEvent(MenuEvent),
    RoomEvent(RoomEvent),
}

pub enum MenuItem {
    CreateRoom,
}

pub enum MenuEvent {
    Cancel,
    CreateRoom(RoomBrief),
}

pub enum RoomEvent {
    Quit,
    Join(RoomId),
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, mut link: ComponentLink<Self>) -> Self {
        link.send_self(Msg::Connect);
        Model {
            ws: None,
            link,
            socket: WebSocketService::new(),
            connected: None,
            client: None,
            menu: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Connect => {
                let callback = self.link.send_back(|Json(msg)| Msg::WebResponse(msg));
                let notification = self.link.send_back(|status| match status {
                    WebSocketStatus::Opened => Msg::Connected,
                    _ => Msg::Failed,
                });
                self.ws = Some(self.socket.connect(
                    "ws://127.0.0.1:8080/ws/",
                    callback,
                    notification,
                ));
                false
            }
            Msg::Connected => {
                self.connected = Some(true);
                true
            }
            Msg::Failed => {
                self.connected = Some(false);
                true
            }
            Msg::SendChat(text) => {
                self.link
                    .send_self(Msg::WebRequest(bridge::client::Message::Chat {
                        text,
                        to: None,
                    }));
                false
            }
            Msg::OpenMenu(menu) => {
                self.menu = Some(menu);
                true
            }
            Msg::MenuEvent(event) => {
                match event {
                    MenuEvent::Cancel => {
                        self.menu = None;
                        true
                    }
                    MenuEvent::CreateRoom(room) => {
                        self.menu = None;
                        self.link.send_self(Msg::WebRequest(bridge::client::Message::CreateRoom {
                            room,
                        }));
                        true
                    }
                }
            }
            Msg::RoomEvent(event) => {
                match event {
                    RoomEvent::Quit => {
                        let client = self.client.as_mut().unwrap();
                        client.room = None;
                        self.link.send_self(Msg::WebRequest(bridge::client::Message::QuitRoom));
                        true
                    }
                    RoomEvent::Join(room) => {
                        self.link.send_self(Msg::WebRequest(bridge::client::Message::JoinRoom { room }));
                        false
                    }
                }
            }
            Msg::WebRequest(message) => {
                let ws = self.ws.as_mut().unwrap();
                ws.send_binary(Json(&message));
                false
            }
            Msg::WebResponse(Ok(message)) => {
                use bridge::server::Message;
                match message {
                    Message::Welcome { id, users, rooms } => {
                        let user_list = users.iter().fold(HashMap::new(), |mut result, user| {
                            result.insert(user.id, user.clone());
                            result
                        });
                        let room_list = rooms.iter().fold(HashMap::new(), |mut result, room| {
                            result.insert(room.id, room.clone());
                            result
                        });
                        self.client = Some(Client {
                            id,
                            users: user_list,
                            chats: vec![],
                            rooms: room_list,
                            room: None,
                        });
                        true
                    }
                    Message::Connected { user } => {
                        let client = self.client.as_mut().unwrap();
                        client.users.insert(user.id, user);
                        true
                    }
                    Message::Disconnected { id } => {
                        let client = self.client.as_mut().unwrap();
                        client.users.remove(&id);
                        true
                    }
                    Message::Alert(alert) => {
                        let client = self.client.as_mut().unwrap();
                        client.chats.push(Chat::Alert(
                            match alert {
                                Alert::TargetNotFound => {
                                    "상대방이 접속 중이지 않습니다.".into()
                                }
                                Alert::Join { user } => {
                                    format!("{}님이 입장하셨습니다.", client.users.get(&user).map_or("(정보 없음)".into(), |user| user.name.clone()))
                                }
                                Alert::Quit { user } => {
                                    format!("{}님이 퇴장하셨습니다.", client.users.get(&user).map_or("(정보 없음)".into(), |user| user.name.clone()))
                                }
                            },
                        ));
                        true
                    }
                    Message::Chat {
                        from,
                        text,
                        whisper,
                    } => {
                        let client = self.client.as_mut().unwrap();
                        client.chats.push(if whisper {
                            Chat::Whisper(text, from)
                        } else {
                            Chat::Chat(text, from)
                        });
                        true
                    }
                    // 로비에서만 업데이트
                    Message::NewRoom {
                        room
                    } => {
                        let client = self.client.as_mut().unwrap();
                        client.rooms.insert(room.id, room);
                        client.room.is_none()
                    }
                    Message::DestroyRoom {
                        room
                    } => {
                        let client = self.client.as_mut().unwrap();
                        client.rooms.remove(&room);
                        client.room.is_none()
                    }
                    Message::ReadyJoin {
                        room
                    } => {
                        let client = self.client.as_mut().unwrap();
                        client.room = Some(room);
                        true
                    }
                    Message::RoomUpdate {
                        room
                    } => {
                        let client = self.client.as_mut().unwrap();
                        client.room = Some(room);
                        true
                    }
                    Message::RoomDetail { .. } => unimplemented!(),
                }
            }
            Msg::WebResponse(Err(_)) => false,
        }
    }
}

impl Renderable<Model> for Model {
    fn view(&self) -> Html<Self> {
        let body = if let Some(client) = &self.client {
            let users = &client.users;
            let chats = client.chats.clone().into_boxed_slice();
            let main = if let Some(room) = &client.room {
                let mut members = room.members
                    .iter()
                    .filter_map(|user| client.users.get(&user))
                    .map(|user| html! {
                        <li>
                            <a href="#",>
                                <header>{ &user.name }</header>
                                {
                                    if user.id == room.owner {
                                        html! { <i>{ "방장" }</i> }
                                    } else {
                                        html! {}
                                    }
                                }
                            </a>
                        </li>
                    });
                html! {
                    <>
                        <section id="menu",>
                            <ul>
                                <li><a href="#", onclick=|_| Msg::RoomEvent(RoomEvent::Quit)>{ "나가기" }</a></li>
                            </ul>
                        </section>
                        <section id="room-members",>
                            <ul>
                                { for members }
                            </ul>
                        </section>
                    </>
                }
            } else {
                html! {
                    <>
                        <section id="menu",>
                            <ul>
                                <li><a href="#", onclick=|_| Msg::OpenMenu(MenuItem::CreateRoom)>{ "방 만들기" }</a></li>
                            </ul>
                        </section>
                        <section id="room-list",>
                            <RoomList: rooms=&client.rooms, onclick=|room_id| Msg::RoomEvent(RoomEvent::Join(room_id)),/>
                        </section>
                    </>
                }
            };
            html! {
                <>
                <section>
                    <aside id="user-list">
                        <UserList: users=users,/>
                    </aside>
                    <main>{ main }</main>
                </section>
                <footer>
                    <aside>
                    </aside>
                    <article id="chat-box",>
                        <ChatBox: chats=chats, mapper=users, onsubmit=|text| Msg::SendChat(text),/>
                    </article>
                </footer>
                </>
            }
        } else {
            html! {}
        };
        html! {
            <>
                <section id="game-area",>{ body }</section>
                <section id="modal-area",>
                {
                    if let Some(item) = &self.menu {
                        match item {
                            MenuItem::CreateRoom => html! {
                                <CreateRoomModal: onsubmit=|room| Msg::MenuEvent(MenuEvent::CreateRoom(room)),
                                                  oncancel=|_| Msg::MenuEvent(MenuEvent::Cancel), />
                            }
                        }
                    } else {
                        html! {}
                    }
                }
                </section>
            </>
        }
    }
}
