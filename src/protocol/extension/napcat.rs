// https://napneko.github.io
// https://napcat.apifox.cn

use crate::protocol::{Protocol, adapter::Response, message::Message};
use anyhow::Result;
use serde_json::json;

impl Protocol {
    /// 发送合并转发
    ///
    /// # 参数
    ///
    /// * `message_type` - 消息类型，支持 `private`、`group`，分别对应私聊、群组，
    ///                   如不传入，则根据传入的 `*_id` 参数判断
    /// * `user_id` - 对方 QQ 号（消息类型为 `private` 时需要）
    /// * `group_id` - 群号（消息类型为 `group` 时需要）
    /// * `message` - 消息,需要是 node[], 详见 node
    ///
    /// # 响应数据
    ///
    /// * `message_id` - 消息 ID (number int32)
    /// * `res_id` - resid (String)
    pub async fn send_forward_msg(
        &self,
        message_type: Option<&str>,
        user_id: Option<i64>,
        group_id: Option<i64>,
        message: &Message,
    ) -> Result<Response> {
        let mut data = json!({"message": message});
        if let Some(msg_type) = message_type {
            data["message_type"] = json!(msg_type);
        }
        if let Some(uid) = user_id {
            data["user_id"] = json!(uid);
        }
        if let Some(gid) = group_id {
            data["group_id"] = json!(gid);
        }
        self.send_request("send_msg", data).await
    }
}
