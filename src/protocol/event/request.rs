use super::EventBase;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "request_type", rename_all = "snake_case")]
#[allow(unused)]
pub enum Request {
    Friend(FriendRequest),
    Group(GroupRequest),
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct FriendRequest {
    #[serde(flatten)]
    pub base: EventBase,
    pub user_id: i64,
    pub comment: String,
    pub flag: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct GroupRequest {
    #[serde(flatten)]
    pub base: EventBase,
    pub sub_type: GroupRequestType,
    pub group_id: i64,
    pub user_id: i64,
    pub comment: String,
    pub flag: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(unused)]
pub enum GroupRequestType {
    Add,
    Invite,
}
