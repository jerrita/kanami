use serde::Deserialize;

mod message;
mod meta;
mod notice;
mod request;

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct EventBase {
    pub time: i64,
    pub self_id: i64,
}

#[allow(unused)]
pub use message::*;
#[allow(unused)]
pub use meta::*;
#[allow(unused)]
pub use notice::*;
#[allow(unused)]
pub use request::*;

#[derive(Debug, Deserialize)]
#[serde(tag = "post_type", rename_all = "snake_case")]
#[allow(unused)]
// Keep the existing event representation and OneBot event names.
#[allow(clippy::large_enum_variant, clippy::enum_variant_names)]
pub enum Event {
    #[serde(rename = "message")]
    MessageEvent(message::MessageEvent),
    Notice(notice::Notice),
    #[serde(rename = "request")]
    RequestEvent(request::Request),
    MetaEvent(meta::MetaEvent),
}

impl Event {
    pub fn group_id(&self) -> Option<i64> {
        match self {
            Self::MessageEvent(MessageEvent::Group(event)) => Some(event.group_id),
            Self::MessageEvent(MessageEvent::Private(_)) | Self::MetaEvent(_) => None,
            Self::RequestEvent(Request::Group(event)) => Some(event.group_id),
            Self::RequestEvent(Request::Friend(_)) => None,
            Self::Notice(event) => match event {
                Notice::GroupUpload(event) => Some(event.group_id),
                Notice::GroupAdmin(event) => Some(event.group_id),
                Notice::GroupDecrease(event) => Some(event.group_id),
                Notice::GroupIncrease(event) => Some(event.group_id),
                Notice::GroupBan(event) => Some(event.group_id),
                Notice::GroupRecall(event) => Some(event.group_id),
                Notice::FriendAdd(_) | Notice::FriendRecall(_) => None,
                Notice::Notify(event) => match event {
                    NotifyEvent::Poke(event) => Some(event.group_id),
                    NotifyEvent::LuckyKing(event) => Some(event.group_id),
                    NotifyEvent::Honor(event) => Some(event.group_id),
                },
                #[cfg(feature = "napcat")]
                Notice::GroupCard(event) => Some(event.group_id),
            },
        }
    }

    pub fn user_id(&self) -> Option<i64> {
        match self {
            Self::MessageEvent(event) => Some(event.user_id()),
            Self::RequestEvent(Request::Group(event)) => Some(event.user_id),
            Self::RequestEvent(Request::Friend(event)) => Some(event.user_id),
            Self::MetaEvent(_) => None,
            Self::Notice(event) => match event {
                Notice::GroupUpload(event) => Some(event.user_id),
                Notice::GroupAdmin(event) => Some(event.user_id),
                Notice::GroupDecrease(event) => Some(event.user_id),
                Notice::GroupIncrease(event) => Some(event.user_id),
                Notice::GroupBan(event) => Some(event.user_id),
                Notice::GroupRecall(event) => Some(event.user_id),
                Notice::FriendAdd(event) => Some(event.user_id),
                Notice::FriendRecall(event) => Some(event.user_id),
                Notice::Notify(event) => match event {
                    NotifyEvent::Poke(event) => Some(event.user_id),
                    NotifyEvent::LuckyKing(event) => Some(event.user_id),
                    NotifyEvent::Honor(event) => Some(event.user_id),
                },
                #[cfg(feature = "napcat")]
                Notice::GroupCard(event) => Some(event.user_id),
            },
        }
    }
}
