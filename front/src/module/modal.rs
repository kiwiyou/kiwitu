use yew::prelude::*;
use bridge::*;

mod create_room {
    use super::*;

    #[derive(Clone, PartialEq, Default)]
    pub struct CreateRoomModalProps {
        pub onsubmit: Option<Callback<RoomBrief>>,
        pub oncancel: Option<Callback<()>>,
    }
    pub struct CreateRoomModal {
        title: String,
        onsubmit: Option<Callback<RoomBrief>>,
        oncancel: Option<Callback<()>>,
    }
    pub enum Msg {
        Submit,
        Cancelled,
        GotInput(String),
    }

    impl Component for CreateRoomModal {
        type Message = Msg;
        type Properties = CreateRoomModalProps;

        fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
            Self {
                title: String::new(),
                onsubmit: props.onsubmit,
                oncancel: props.oncancel,
            }
        }

        fn update(&mut self, msg: Self::Message) -> ShouldRender {
            match msg {
                Msg::Submit => {
                    if let Some(onsubmit) = &self.onsubmit {
                        let title = self.title.trim().to_string();
                        if title.len() > 0 {
                            onsubmit.emit(RoomBrief {
                                id: 0,
                                title,
                            });
                        }
                    }
                    false
                }
                Msg::Cancelled => {
                    if let Some(oncancel) = &self.oncancel {
                        oncancel.emit(());
                    }
                    false
                }
                Msg::GotInput(new_title) => {
                    self.title = new_title;
                    true
                }
            }
        }
    }

    impl Renderable<CreateRoomModal> for CreateRoomModal {
        fn view(&self) -> Html<Self> {
            html! {
                <dialog open=true>
                    <form action="javascript:void(0)", onsubmit=|_| Msg::Submit,>
                        <label for="title",>{ "방 제목" }</label>
                        <input type="text", name="title", value=self.title, oninput=|e| Msg::GotInput(e.value),/>
                        <fieldset>
                            <input type="button", value="취소", onclick=|_| Msg::Cancelled,/>
                            <input type="submit", value="확인",/>
                        </fieldset>
                    </form>
                </dialog>
            }
        }
    }
}

pub use create_room::{CreateRoomModal, CreateRoomModalProps};