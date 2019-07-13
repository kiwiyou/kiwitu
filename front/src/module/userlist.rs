use bridge::*;
use std::collections::HashMap;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub struct UserListProps {
    pub users: HashMap<UserId, UserBrief>,
    pub onclick: Option<Callback<UserId>>,
}

pub enum Msg {
    Clicked(UserId),
}

pub struct UserList {
    users: HashMap<UserId, UserBrief>,
    onclick: Option<Callback<UserId>>,
}

impl Default for UserListProps {
    fn default() -> Self {
        Self {
            users: HashMap::new(),
            onclick: None,
        }
    }
}

impl Component for UserList {
    type Message = Msg;
    type Properties = UserListProps;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Self {
            users: props.users,
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
        self.users = props.users;
        self.onclick = props.onclick;
        true
    }
}

impl Renderable<UserList> for UserList {
    fn view(&self) -> Html<Self> {
        let mut list = self.users.values().cloned().map(|user| {
            let UserBrief { id, name, .. } = user;
            html! {
                <li><a href="#", onclick=|_| Msg::Clicked(id),>{ &name }</a></li>
            }
        });
        html! {
            <ul>{ for list }</ul>
        }
    }
}
