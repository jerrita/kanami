use crate::{
    config,
    protocol::{
        event::{GroupRole, MessageEvent},
        message::{Message, Segment},
    },
};
use serde::{Deserialize, Serialize};

/// GSCore 消息类型
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum GSCoreMessage {
    Log(String),
    Text(String),
    Markdown(String),
    Image(String),
    At(String),
    Reply(String),
    Node(Vec<GSCoreMessageWithoutNode>),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum GSCoreMessageWithoutNode {
    Text(String),
    Markdown(String),
    Image(String),
    At(String),
    Reply(String),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetType {
    Group,
    Direct,
    Channel,
    SubChannel,
}

/// 发给 GSCore 的消息包
///
/// user_type: group, direct, channel, sub_channel
#[derive(Debug, Serialize)]
pub struct MessageReceive {
    bot_id: String,
    bot_self_id: String,
    msg_id: String,
    user_type: TargetType,
    group_id: Option<String>,
    user_id: String,
    sender: MessageSender,
    user_pm: i32,
    content: Vec<GSCoreMessage>,
}

#[derive(Debug, Serialize)]
pub struct MessageSender {
    pub nickname: String,
    pub avatar: String,
}

/// 从 GSCore 收到的消息包
#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct MessageSend {
    pub bot_id: String,
    pub bot_self_id: String,
    pub msg_id: String,
    pub target_type: Option<TargetType>,
    pub target_id: Option<String>,
    pub content: Option<Vec<GSCoreMessage>>,
}

impl From<&MessageEvent> for MessageReceive {
    fn from(value: &MessageEvent) -> Self {
        match value {
            MessageEvent::Group(event) => Self {
                bot_id: config::GSCORE_BOTID.to_string(),
                bot_self_id: event.base.self_id.to_string(),
                msg_id: event.message_id.to_string(),
                user_type: TargetType::Group,
                group_id: Some(event.group_id.to_string()),
                user_id: event.user_id.to_string(),
                sender: MessageSender {
                    nickname: event.sender.nickname.to_string(),
                    avatar: format!(
                        "http://q.qlogo.cn/headimg_dl?dst_uin={}&spec=640&img_type=jpg",
                        event.user_id
                    ),
                },
                user_pm: match event.sender.role {
                    Some(GroupRole::Owner) => 2,
                    Some(GroupRole::Admin) => 3,
                    _ => 6,
                },
                content: (&event.message).into(),
            },
            MessageEvent::Private(event) => Self {
                bot_id: config::GSCORE_BOTID.to_string(),
                bot_self_id: event.base.self_id.to_string(),
                msg_id: event.message_id.to_string(),
                user_type: TargetType::Direct,
                group_id: None,
                user_id: event.user_id.to_string(),
                sender: MessageSender {
                    nickname: event.sender.nickname.to_string(),
                    avatar: format!(
                        "http://q.qlogo.cn/headimg_dl?dst_uin={}&spec=640&img_type=jpg",
                        event.user_id
                    ),
                },
                user_pm: 6,
                content: (&event.message).into(),
            },
        }
    }
}

impl From<&Segment> for GSCoreMessage {
    fn from(value: &Segment) -> Self {
        match value {
            Segment::Text { text } => GSCoreMessage::Text(text.clone()),
            Segment::Image {
                file,
                catagary: _,
                url: _,
                cache: _,
                proxy: _,
                timeout: _,
            } => GSCoreMessage::Image(file.clone()),
            Segment::At { qq } => GSCoreMessage::At(qq.clone()),
            Segment::Reply { id } => GSCoreMessage::Reply(id.clone()),
            _ => GSCoreMessage::Text(format!("<unsupp: {:?}>", value)),
        }
    }
}

impl From<&GSCoreMessage> for Segment {
    fn from(value: &GSCoreMessage) -> Self {
        match value {
            GSCoreMessage::Text(text) => Segment::Text { text: text.clone() },
            GSCoreMessage::Markdown(text) => Segment::Text { text: text.clone() },
            GSCoreMessage::Image(url) => Segment::Image {
                file: url.clone(),
                catagary: None,
                url: None,
                cache: None,
                proxy: None,
                timeout: None,
            },
            GSCoreMessage::At(qq) => Segment::At { qq: qq.clone() },
            GSCoreMessage::Reply(id) => Segment::Reply { id: id.clone() },
            GSCoreMessage::Node(node) => Segment::Node {
                id: None,
                user_id: Some(config::GSCORE_NODE_SENDER_ID.to_string()),
                nickname: Some(config::GSCORE_NODE_SENDER_NICKNAME.to_string()),
                content: Some(
                    node.iter()
                        .map(|seg| seg.into())
                        .collect::<Vec<Segment>>()
                        .into(),
                ),
            },
            _ => Segment::Text {
                text: format!("<unsupp: {:?}>", value),
            },
        }
    }
}

impl From<&GSCoreMessageWithoutNode> for Segment {
    fn from(value: &GSCoreMessageWithoutNode) -> Self {
        match value {
            GSCoreMessageWithoutNode::Text(text) => Segment::Text { text: text.clone() },
            GSCoreMessageWithoutNode::Markdown(text) => Segment::Text { text: text.clone() },
            GSCoreMessageWithoutNode::Image(url) => Segment::Image {
                file: url.clone(),
                catagary: None,
                url: None,
                cache: None,
                proxy: None,
                timeout: None,
            },
            GSCoreMessageWithoutNode::At(qq) => Segment::At { qq: qq.clone() },
            GSCoreMessageWithoutNode::Reply(id) => Segment::Reply { id: id.clone() },
        }
    }
}

impl From<&Vec<GSCoreMessage>> for Message {
    fn from(value: &Vec<GSCoreMessage>) -> Self {
        let msg: Vec<Segment> = value.iter().map(|e| e.into()).collect();
        msg.into()
    }
}

impl From<&Message> for Vec<GSCoreMessage> {
    fn from(value: &Message) -> Self {
        value.0.iter().map(|seg| seg.into()).collect()
    }
}
