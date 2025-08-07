use super::EventBase;
use crate::protocol::{
    adapter::Response,
    get_bot,
    message::{self, Message},
};
use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "message_type", rename_all = "snake_case")]
pub enum MessageEvent {
    Private(PrivateMessage),
    Group(GroupMessage),
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct PrivateMessage {
    #[serde(flatten)]
    pub base: EventBase,
    pub sub_type: PrivateMessageType,
    pub message_id: i32,
    pub user_id: i64,
    pub message: message::Message,
    pub raw_message: String,
    pub font: i32,
    pub sender: PrivateSender,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(unused)]
pub enum PrivateMessageType {
    Friend,
    Group,
    Other,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct PrivateSender {
    pub user_id: i64,
    pub nickname: String,
    pub sex: Option<String>,
    pub age: Option<i32>,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct GroupMessage {
    #[serde(flatten)]
    pub base: EventBase,
    pub sub_type: String,
    pub message_id: i32,
    pub group_id: i64,
    pub user_id: i64,
    pub anonymous: Option<Anonymous>,
    pub message: message::Message,
    pub raw_message: String,
    pub font: i32,
    pub sender: GroupSender,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(unused)]
pub enum GroupMessageType {
    Normal,
    Anonymous,
    Notice,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct GroupSender {
    pub user_id: i64,
    pub nickname: String,
    pub card: Option<String>,
    pub sex: Option<String>,
    pub age: Option<i32>,
    pub area: Option<String>,
    pub level: Option<String>,
    pub role: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Anonymous {
    pub id: i64,
    pub name: String,
    pub flag: String,
}

impl PrivateMessage {
    #[allow(unused)]
    pub async fn reply<T>(&self, message: T, quote: bool) -> Result<Response>
    where
        T: Into<Message>,
    {
        let mut message = message.into();
        if quote {
            message.insert(
                0,
                message::Segment::Reply {
                    id: self.message_id.to_string(),
                },
            );
        }
        get_bot()
            .await
            .send_private_message(self.user_id, message)
            .await
    }
}

impl GroupMessage {
    #[allow(unused)]
    pub async fn reply<T>(&self, message: T, quote: bool) -> Result<Response>
    where
        T: Into<Message>,
    {
        let mut message = message.into();
        if quote {
            message.insert(
                0,
                message::Segment::Reply {
                    id: self.message_id.to_string(),
                },
            );
        }
        get_bot()
            .await
            .send_group_message(self.group_id, message)
            .await
    }
}
