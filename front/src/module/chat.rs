use bridge::{UserBrief, UserId};
use yew::prelude::*;

pub enum Msg {
    GotInput(String),
    Submit,
}

use std::collections::HashMap;

#[derive(Clone, PartialEq)]
pub struct ChatBoxProps {
    pub chats: Box<[Chat]>,
    pub mapper: HashMap<UserId, UserBrief>,
    pub onsubmit: Option<Callback<(String)>>,
}

impl Default for ChatBoxProps {
    fn default() -> Self {
        ChatBoxProps {
            chats: vec![].into_boxed_slice(),
            mapper: HashMap::new(),
            onsubmit: None,
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum Chat {
    Alert(String),
    Chat(String, UserId),
    Whisper(String, UserId),
}

pub struct ChatBox {
    chats: Box<[Chat]>,
    mapper: HashMap<UserId, UserBrief>,
    onsubmit: Option<Callback<(String)>>,
    input: String,
}

impl Component for ChatBox {
    type Message = Msg;
    type Properties = ChatBoxProps;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self {
            chats: props.chats,
            onsubmit: props.onsubmit,
            mapper: props.mapper,
            input: String::new(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::GotInput(new_input) => {
                self.input = new_input;
                true
            }
            Msg::Submit => {
                if let Some(callback) = &self.onsubmit {
                    let text = self.input.clone();
                    if text.trim().len() > 0 {
                        callback.emit(text);
                    }
                }
                self.input.clear();
                false
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.chats = props.chats;
        self.mapper = props.mapper;
        self.onsubmit = props.onsubmit;
        true
    }
}

impl Renderable<ChatBox> for ChatBox {
    fn view(&self) -> Html<Self> {
        let mut chat_list = self.chats.iter().map(|chat| {
            let (header, main) = match chat {
                Chat::Alert(text) => ("알림".into(), text),
                Chat::Chat(text, user) | Chat::Whisper(text, user) => {
                    (self.mapper.get(&user).unwrap().name.clone(), text)
                }
            };
            html! {
                <li class="chat-item",>
                    <header>{ header }</header>
                    <main>{ main }</main>
                </li>
            }
        });
        html! {
            <>
                <ul id="chat-list",>{ for chat_list }</ul>
                <form id="chat-input", action="javascript:void(0)", onsubmit=|_| Msg::Submit,>
                    <input type="text", value=self.input, oninput=|e| Msg::GotInput(e.value),/>
                    <input type="submit", value="전송"/>
                </form>
            </>
        }
    }
}
