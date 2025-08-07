use super::EventBase;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
#[allow(unused)]
#[serde(tag = "meta_event_type", rename_all = "lowercase")]
pub enum MetaEvent {
    LifeCycle(LifeCycle),
    HeartBeat(HeartBeat),
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct LifeCycle {
    #[serde(flatten)]
    pub base: EventBase,
    pub sub_type: String,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct HeartBeat {
    #[serde(flatten)]
    pub base: EventBase,
    pub status: Value,
}
