use super::EventBase;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "notice_type", rename_all = "snake_case")]
#[allow(unused)]
pub enum Notice {
    GroupUpload(GroupUploadNotice),
    GroupAdmin(GroupAdminNotice),
    GroupDecrease(GroupDecreaseNotice),
    GroupIncrease(GroupIncreaseNotice),
    GroupBan(GroupBanNotice),
    FriendAdd(FriendAddNotice),
    GroupRecall(GroupRecallNotice),
    FriendRecall(FriendRecallNotice),
    Notify(NotifyEvent),

    #[cfg(feature = "napcat")]
    GroupCard(GroupCardNotice),
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct GroupUploadNotice {
    #[serde(flatten)]
    pub base: EventBase,
    pub group_id: i64,
    pub user_id: i64,
    pub file: File,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct File {
    pub id: String,
    pub name: String,
    pub size: i64,
    pub busid: i64,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct GroupAdminNotice {
    #[serde(flatten)]
    pub base: EventBase,
    pub sub_type: GroupAdminType,
    pub group_id: i64,
    pub user_id: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(unused)]
pub enum GroupAdminType {
    Set,
    Unset,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct GroupDecreaseNotice {
    #[serde(flatten)]
    pub base: EventBase,
    pub sub_type: GroupDecreaseType,
    pub group_id: i64,
    pub operator_id: i64,
    pub user_id: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(unused)]
pub enum GroupDecreaseType {
    Leave,
    Kick,
    KickMe,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct GroupIncreaseNotice {
    #[serde(flatten)]
    pub base: EventBase,
    pub sub_type: GroupIncreaseType,
    pub group_id: i64,
    pub operator_id: i64,
    pub user_id: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(unused)]
pub enum GroupIncreaseType {
    Approve,
    Invite,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct GroupBanNotice {
    #[serde(flatten)]
    pub base: EventBase,
    pub sub_type: GroupBanType,
    pub group_id: i64,
    pub operator_id: i64,
    pub user_id: i64,
    pub duration: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(unused)]
pub enum GroupBanType {
    Ban,
    LiftBan,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct FriendAddNotice {
    #[serde(flatten)]
    pub base: EventBase,
    pub user_id: i64,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct GroupRecallNotice {
    #[serde(flatten)]
    pub base: EventBase,
    pub group_id: i64,
    pub user_id: i64,
    pub operator_id: i64,
    pub message_id: i64,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct FriendRecallNotice {
    #[serde(flatten)]
    pub base: EventBase,
    pub user_id: i64,
    pub message_id: i64,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "sub_type", rename_all = "snake_case")]
#[allow(unused)]
pub enum NotifyEvent {
    Poke(PokeNotify),
    LuckyKing(LuckyKingNotify),
    Honor(HonorNotify),
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct PokeNotify {
    #[serde(flatten)]
    pub base: EventBase,
    pub group_id: i64,
    pub user_id: i64,
    pub target_id: i64,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct LuckyKingNotify {
    #[serde(flatten)]
    pub base: EventBase,
    pub group_id: i64,
    pub user_id: i64,
    pub target_id: i64,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct HonorNotify {
    #[serde(flatten)]
    pub base: EventBase,
    pub group_id: i64,
    pub honor_type: HonorType,
    pub user_id: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(unused)]
pub enum HonorType {
    Talkative,
    Performer,
    Emotion,
}

#[cfg(feature = "napcat")]
#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct GroupCardNotice {
    #[serde(flatten)]
    pub base: EventBase,
    pub group_id: i64,
    pub user_id: i64,
    pub card_new: String,
    pub card_old: String,
}
