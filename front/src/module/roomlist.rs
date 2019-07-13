use bridge::*;
use std::collections::HashMap;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub struct RoomListProps {
    pub rooms: HashMap<RoomId, RoomBrief>,
    pub onclick: Option<Callback<RoomId>>,
}

pub enum Msg {
    Clicked(RoomId),
}

pub struct RoomList {
    rooms: HashMap<RoomId, RoomBrief>,
    onclick: Option<Callback<RoomId>>,
}

impl Default for RoomListProps {
    fn default() -> Self {
        Self {
            rooms: HashMap::new(),
            onclick: None,
        }
    }
}

impl Component for RoomList {
    type Message = Msg;
    type Properties = RoomListProps;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self {
            rooms: props.rooms,
            onclick: props.onclick,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Clicked(id) => {
                if let Some(onclick) = &self.onclick {
                    onclick.emit(id);
                }
                false
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.rooms = props.rooms;
        self.onclick = props.onclick;
        true
    }
}

impl Renderable<RoomList> for RoomList {
    fn view(&self) -> Html<Self> {
        let mut list = self.rooms.values().cloned().map(|room| {
            let RoomBrief { id, title, .. } = room;
            html! {
                <li>
                    <a href="#", onclick=|_| Msg::Clicked(id),>
                        <header>{ &id }</header>
                        <h1>{ &title }</h1>
                    </a>
                </li>
            }
        });
        html! {
            <ul>{ for list }</ul>
        }
    }
}
