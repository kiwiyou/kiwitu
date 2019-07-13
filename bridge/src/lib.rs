use serde::{Deserialize, Serialize};
pub type UserId = usize;
pub type RoomId = usize;

pub mod client {
    use super::*;

    #[derive(Serialize, Deserialize)]
    pub enum Message {
        Chat { text: String, to: Option<UserId> },
        CreateRoom { room: RoomBrief, },
        GetRoomDetail { room: RoomId },
        JoinRoom { room: RoomId, },
        QuitRoom,
    }
}

pub mod server {
    use super::*;
    #[cfg(feature = "server")]
    use actix::prelude::*;
    #[cfg_attr(feature = "server", derive(Message))]
    #[derive(Serialize, Deserialize)]
    pub enum Message {
        Connected {
            user: UserBrief,
        },
        Welcome {
            id: UserId,
            users: Box<[UserBrief]>,
            rooms: Box<[RoomBrief]>,
        },
        Disconnected {
            id: UserId,
        },
        Alert(Alert),
        Chat {
            from: UserId,
            text: String,
            whisper: bool,
        },
        NewRoom {
            room: RoomBrief,
        },
        DestroyRoom {
            room: RoomId,
        },
        RoomDetail {
            room: Room,
        },
        ReadyJoin {
            room: Room,
        },
        RoomUpdate {
            room: Room,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub enum Alert {
    TargetNotFound,
    Join { user: UserId },
    Quit { user: UserId },
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct UserBrief {
    pub id: UserId,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct RoomBrief {
    pub id: RoomId,
    pub title: String,
}

#[derive(Serialize, Deserialize)]
pub struct Room {
    pub id: RoomId,
    pub title: String,
    pub owner: UserId,
    pub members: Vec<UserId>,
}