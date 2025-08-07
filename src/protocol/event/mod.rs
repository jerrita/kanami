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
pub enum Event {
    #[serde(rename = "message")]
    MessageEvent(message::MessageEvent),
    Notice(notice::Notice),
    #[serde(rename = "request")]
    RequestEvent(request::Request),
    MetaEvent(meta::MetaEvent),
}
