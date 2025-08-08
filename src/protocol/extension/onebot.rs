use crate::protocol::{Protocol, Response, message::Message};
use anyhow::Result;
use serde_json::{Value, json};

#[allow(unused)]
impl Protocol {
    // 消息相关 API

    /// 发送私聊消息
    ///
    /// # 参数
    ///
    /// * `user_id` - 对方 QQ 号
    /// * `message` - 要发送的内容
    ///
    /// # 响应数据
    ///
    /// * `message_id` - 消息 ID (number int32)
    pub async fn send_private_message<T>(&self, user_id: i64, message: T) -> Result<Response>
    where
        T: Into<Message>,
    {
        let message = message.into();
        log::info!("User({}) <- {}", user_id, message);
        let data = json!({"user_id": user_id, "message": message});
        self.send_request("send_private_msg", data).await
    }

    /// 发送群消息
    ///
    /// # 参数
    ///
    /// * `group_id` - 群号
    /// * `message` - 要发送的内容
    ///
    /// # 响应数据
    ///
    /// * `message_id` - 消息 ID (number int32)
    pub async fn send_group_message<T>(&self, group_id: i64, message: T) -> Result<Response>
    where
        T: Into<Message>,
    {
        let message = message.into();
        log::info!("Group({}) <- {}", group_id, message);
        let data = json!({"group_id": group_id, "message": message});
        self.send_request("send_group_msg", data).await
    }

    /// 发送消息
    ///
    /// # 参数
    ///
    /// * `message_type` - 消息类型，支持 `private`、`group`，分别对应私聊、群组，
    ///                   如不传入，则根据传入的 `*_id` 参数判断
    /// * `user_id` - 对方 QQ 号（消息类型为 `private` 时需要）
    /// * `group_id` - 群号（消息类型为 `group` 时需要）
    /// * `message` - 要发送的内容
    ///
    /// # 响应数据
    ///
    /// * `message_id` - 消息 ID (number int32)
    pub async fn send_message<T>(
        &self,
        message_type: Option<&str>,
        user_id: Option<i64>,
        group_id: Option<i64>,
        message: T,
    ) -> Result<Response>
    where
        T: Into<Message>,
    {
        let message = message.into();
        match (message_type, user_id, group_id) {
            (Some("private"), Some(uid), _) | (None, Some(uid), None) => {
                log::info!("User({}) <- {}", uid, message);
            }
            (Some("group"), _, Some(gid)) | (None, _, Some(gid)) => {
                log::info!("Group({}) <- {}", gid, message);
            }
            _ => {
                log::info!("Send message: {}", message);
            }
        }

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

    /// 撤回消息
    ///
    /// # 参数
    ///
    /// * `message_id` - 消息 ID (number int32)
    ///
    /// # 响应数据
    ///
    /// 无
    pub async fn delete_message(&self, message_id: i32) -> Result<Response> {
        let data = json!({"message_id": message_id});
        self.send_request("delete_msg", data).await
    }

    /// 获取消息
    ///
    /// # 参数
    ///
    /// * `message_id` - 消息 ID (number int32)
    ///
    /// # 响应数据
    ///
    /// * `time` - 发送时间 (number int32)
    /// * `message_type` - 消息类型，同消息事件
    /// * `message_id` - 消息 ID (number int32)
    /// * `real_id` - 消息真实 ID (number int32)
    /// * `sender` - 发送人信息，同消息事件
    /// * `message` - 消息内容
    pub async fn get_message(&self, message_id: i32) -> Result<Response> {
        let data = json!({"message_id": message_id});
        self.send_request("get_msg", data).await
    }

    /// 获取合并转发消息
    ///
    /// # 参数
    ///
    /// * `id` - 合并转发 ID
    ///
    /// # 响应数据
    ///
    /// * `message` - 消息内容，使用消息的数组格式表示，数组中的消息段全部为 `node` 消息段
    pub async fn get_forward_message(&self, id: &str) -> Result<Response> {
        let data = json!({"id": id});
        self.send_request("get_forward_msg", data).await
    }

    // 好友相关 API

    /// 发送好友赞
    ///
    /// # 参数
    ///
    /// * `user_id` - 对方 QQ 号
    /// * `times` - 赞的次数，每个好友每天最多 10 次，默认为 1
    ///
    /// # 响应数据
    ///
    /// 无
    pub async fn send_like(&self, user_id: i64, times: i32) -> Result<Response> {
        let data = json!({"user_id": user_id, "times": times});
        self.send_request("send_like", data).await
    }

    // 群组管理 API

    /// 群组踢人
    ///
    /// # 参数
    ///
    /// * `group_id` - 群号
    /// * `user_id` - 要踢的 QQ 号
    /// * `reject_add_request` - 拒绝此人的加群请求，默认为 `false`
    ///
    /// # 响应数据
    ///
    /// 无
    pub async fn set_group_kick(
        &self,
        group_id: i64,
        user_id: i64,
        reject_add_request: bool,
    ) -> Result<Response> {
        let data = json!({"group_id": group_id, "user_id": user_id, "reject_add_request": reject_add_request});
        self.send_request("set_group_kick", data).await
    }

    /// 群组单人禁言
    ///
    /// # 参数
    ///
    /// * `group_id` - 群号
    /// * `user_id` - 要禁言的 QQ 号
    /// * `duration` - 禁言时长，单位秒，0 表示取消禁言，默认为 30*60
    ///
    /// # 响应数据
    ///
    /// 无
    pub async fn set_group_ban(
        &self,
        group_id: i64,
        user_id: i64,
        duration: i32,
    ) -> Result<Response> {
        let data = json!({"group_id": group_id, "user_id": user_id, "duration": duration});
        self.send_request("set_group_ban", data).await
    }

    /// 群组匿名用户禁言
    ///
    /// # 参数
    ///
    /// * `group_id` - 群号
    /// * `anonymous` - 可选，要禁言的匿名用户对象（群消息上报的 `anonymous` 字段）
    /// * `anonymous_flag` - 可选，要禁言的匿名用户的 flag（需从群消息上报的数据中获得）
    /// * `duration` - 禁言时长，单位秒，无法取消匿名用户禁言，默认为 30*60
    ///
    /// # 说明
    ///
    /// `anonymous` 和 `anonymous_flag` 两者任选其一传入即可，若都传入，则使用 `anonymous`
    ///
    /// # 响应数据
    ///
    /// 无
    pub async fn set_group_anonymous_ban(
        &self,
        group_id: i64,
        anonymous: Option<Value>,
        anonymous_flag: Option<&str>,
        duration: i32,
    ) -> Result<Response> {
        let mut data = json!({"group_id": group_id, "duration": duration});
        if let Some(anon) = anonymous {
            data["anonymous"] = anon;
        }
        if let Some(flag) = anonymous_flag {
            data["anonymous_flag"] = json!(flag);
        }
        self.send_request("set_group_anonymous_ban", data).await
    }

    /// 群组全员禁言
    ///
    /// # 参数
    ///
    /// * `group_id` - 群号
    /// * `enable` - 是否禁言，默认为 `true`
    ///
    /// # 响应数据
    ///
    /// 无
    pub async fn set_group_whole_ban(&self, group_id: i64, enable: bool) -> Result<Response> {
        let data = json!({"group_id": group_id, "enable": enable});
        self.send_request("set_group_whole_ban", data).await
    }

    /// 群组设置管理员
    ///
    /// # 参数
    ///
    /// * `group_id` - 群号
    /// * `user_id` - 要设置管理员的 QQ 号
    /// * `enable` - true 为设置，false 为取消，默认为 `true`
    ///
    /// # 响应数据
    ///
    /// 无
    pub async fn set_group_admin(
        &self,
        group_id: i64,
        user_id: i64,
        enable: bool,
    ) -> Result<Response> {
        let data = json!({"group_id": group_id, "user_id": user_id, "enable": enable});
        self.send_request("set_group_admin", data).await
    }

    /// 群组匿名
    ///
    /// # 参数
    ///
    /// * `group_id` - 群号
    /// * `enable` - 是否允许匿名聊天，默认为 `true`
    ///
    /// # 响应数据
    ///
    /// 无
    pub async fn set_group_anonymous(&self, group_id: i64, enable: bool) -> Result<Response> {
        let data = json!({"group_id": group_id, "enable": enable});
        self.send_request("set_group_anonymous", data).await
    }

    /// 设置群名片（群备注）
    ///
    /// # 参数
    ///
    /// * `group_id` - 群号
    /// * `user_id` - 要设置的 QQ 号
    /// * `card` - 群名片内容，不填或空字符串表示删除群名片
    ///
    /// # 响应数据
    ///
    /// 无
    pub async fn set_group_card(
        &self,
        group_id: i64,
        user_id: i64,
        card: &str,
    ) -> Result<Response> {
        let data = json!({"group_id": group_id, "user_id": user_id, "card": card});
        self.send_request("set_group_card", data).await
    }

    /// 设置群名
    ///
    /// # 参数
    ///
    /// * `group_id` - 群号 (number int64)
    /// * `group_name` - 新群名
    ///
    /// # 响应数据
    ///
    /// 无
    pub async fn set_group_name(&self, group_id: i64, group_name: &str) -> Result<Response> {
        let data = json!({"group_id": group_id, "group_name": group_name});
        self.send_request("set_group_name", data).await
    }

    /// 退出群组
    ///
    /// # 参数
    ///
    /// * `group_id` - 群号
    /// * `is_dismiss` - 是否解散，如果登录号是群主，则仅在此项为 true 时能够解散，默认为 `false`
    ///
    /// # 响应数据
    ///
    /// 无
    pub async fn set_group_leave(&self, group_id: i64, is_dismiss: bool) -> Result<Response> {
        let data = json!({"group_id": group_id, "is_dismiss": is_dismiss});
        self.send_request("set_group_leave", data).await
    }

    /// 设置群组专属头衔
    ///
    /// # 参数
    ///
    /// * `group_id` - 群号
    /// * `user_id` - 要设置的 QQ 号
    /// * `special_title` - 专属头衔，不填或空字符串表示删除专属头衔
    /// * `duration` - 专属头衔有效期，单位秒，-1 表示永久，默认为 `-1`
    ///
    /// # 说明
    ///
    /// duration 参数似乎没有效果，可能是只有某些特殊的时间长度有效，有待测试
    ///
    /// # 响应数据
    ///
    /// 无
    pub async fn set_group_special_title(
        &self,
        group_id: i64,
        user_id: i64,
        special_title: &str,
        duration: i32,
    ) -> Result<Response> {
        let data = json!({"group_id": group_id, "user_id": user_id, "special_title": special_title, "duration": duration});
        self.send_request("set_group_special_title", data).await
    }

    // 请求处理 API

    /// 处理加好友请求
    ///
    /// # 参数
    ///
    /// * `flag` - 加好友请求的 flag（需从上报的数据中获得）
    /// * `approve` - 是否同意请求，默认为 `true`
    /// * `remark` - 添加后的好友备注（仅在同意时有效）
    ///
    /// # 响应数据
    ///
    /// 无
    pub async fn set_friend_add_request(
        &self,
        flag: &str,
        approve: bool,
        remark: &str,
    ) -> Result<Response> {
        let data = json!({"flag": flag, "approve": approve, "remark": remark});
        self.send_request("set_friend_add_request", data).await
    }

    /// 处理加群请求／邀请
    ///
    /// # 参数
    ///
    /// * `flag` - 加群请求的 flag（需从上报的数据中获得）
    /// * `sub_type` - `add` 或 `invite`，请求类型（需要和上报消息中的 `sub_type` 字段相符）
    /// * `approve` - 是否同意请求／邀请，默认为 `true`
    /// * `reason` - 拒绝理由（仅在拒绝时有效）
    ///
    /// # 响应数据
    ///
    /// 无
    pub async fn set_group_add_request(
        &self,
        flag: &str,
        sub_type: &str,
        approve: bool,
        reason: &str,
    ) -> Result<Response> {
        let data =
            json!({"flag": flag, "sub_type": sub_type, "approve": approve, "reason": reason});
        self.send_request("set_group_add_request", data).await
    }

    // 信息获取 API

    /// 获取登录号信息
    ///
    /// # 参数
    ///
    /// 无
    ///
    /// # 响应数据
    ///
    /// * `user_id` - QQ 号 (number int64)
    /// * `nickname` - QQ 昵称
    pub async fn get_login_info(&self) -> Result<Response> {
        self.send_request("get_login_info", json!({})).await
    }

    /// 获取陌生人信息
    ///
    /// # 参数
    ///
    /// * `user_id` - QQ 号
    /// * `no_cache` - 是否不使用缓存（使用缓存可能更新不及时，但响应更快），默认为 `false`
    ///
    /// # 响应数据
    ///
    /// * `user_id` - QQ 号 (number int64)
    /// * `nickname` - 昵称
    /// * `sex` - 性别，`male` 或 `female` 或 `unknown`
    /// * `age` - 年龄 (number int32)
    pub async fn get_stranger_info(&self, user_id: i64, no_cache: bool) -> Result<Response> {
        let data = json!({"user_id": user_id, "no_cache": no_cache});
        self.send_request("get_stranger_info", data).await
    }

    /// 获取好友列表
    ///
    /// # 参数
    ///
    /// 无
    ///
    /// # 响应数据
    ///
    /// 响应内容为 JSON 数组，每个元素如下：
    /// * `user_id` - QQ 号 (number int64)
    /// * `nickname` - 昵称
    /// * `remark` - 备注名
    pub async fn get_friend_list(&self) -> Result<Response> {
        self.send_request("get_friend_list", json!({})).await
    }

    /// 获取群信息
    ///
    /// # 参数
    ///
    /// * `group_id` - 群号
    /// * `no_cache` - 是否不使用缓存（使用缓存可能更新不及时，但响应更快），默认为 `false`
    ///
    /// # 响应数据
    ///
    /// * `group_id` - 群号 (number int64)
    /// * `group_name` - 群名称
    /// * `member_count` - 成员数 (number int32)
    /// * `max_member_count` - 最大成员数（群容量） (number int32)
    pub async fn get_group_info(&self, group_id: i64, no_cache: bool) -> Result<Response> {
        let data = json!({"group_id": group_id, "no_cache": no_cache});
        self.send_request("get_group_info", data).await
    }

    /// 获取群列表
    ///
    /// # 参数
    ///
    /// 无
    ///
    /// # 响应数据
    ///
    /// 响应内容为 JSON 数组，每个元素和 `get_group_info` 接口相同
    pub async fn get_group_list(&self) -> Result<Response> {
        self.send_request("get_group_list", json!({})).await
    }

    /// 获取群成员信息
    ///
    /// # 参数
    ///
    /// * `group_id` - 群号
    /// * `user_id` - QQ 号
    /// * `no_cache` - 是否不使用缓存（使用缓存可能更新不及时，但响应更快），默认为 `false`
    ///
    /// # 响应数据
    ///
    /// * `group_id` - 群号 (number int64)
    /// * `user_id` - QQ 号 (number int64)
    /// * `nickname` - 昵称
    /// * `card` - 群名片／备注
    /// * `sex` - 性别，`male` 或 `female` 或 `unknown`
    /// * `age` - 年龄 (number int32)
    /// * `area` - 地区
    /// * `join_time` - 加群时间戳 (number int32)
    /// * `last_sent_time` - 最后发言时间戳 (number int32)
    /// * `level` - 成员等级
    /// * `role` - 角色，`owner` 或 `admin` 或 `member`
    /// * `unfriendly` - 是否不良记录成员
    /// * `title` - 专属头衔
    /// * `title_expire_time` - 专属头衔过期时间戳 (number int32)
    /// * `card_changeable` - 是否允许修改群名片
    pub async fn get_group_member_info(
        &self,
        group_id: i64,
        user_id: i64,
        no_cache: bool,
    ) -> Result<Response> {
        let data = json!({"group_id": group_id, "user_id": user_id, "no_cache": no_cache});
        self.send_request("get_group_member_info", data).await
    }

    /// 获取群成员列表
    ///
    /// # 参数
    ///
    /// * `group_id` - 群号 (number int64)
    ///
    /// # 响应数据
    ///
    /// 响应内容为 JSON 数组，每个元素的内容和 `get_group_member_info` 接口相同，
    /// 但对于同一个群组的同一个成员，获取列表时和获取单独的成员信息时，某些字段可能有所不同，
    /// 例如 `area`、`title` 等字段在获取列表时无法获得，具体应以单独的成员信息为准
    pub async fn get_group_member_list(&self, group_id: i64) -> Result<Response> {
        let data = json!({"group_id": group_id});
        self.send_request("get_group_member_list", data).await
    }

    /// 获取群荣誉信息
    ///
    /// # 参数
    ///
    /// * `group_id` - 群号 (number int64)
    /// * `honor_type` - 要获取的群荣誉类型，可传入 `talkative` `performer` `legend`
    ///                 `strong_newbie` `emotion` 以分别获取单个类型的群荣誉数据，
    ///                 或传入 `all` 获取所有数据
    ///
    /// # 响应数据
    ///
    /// * `group_id` - 群号 (number int64)
    /// * `current_talkative` - 当前龙王，仅 `type` 为 `talkative` 或 `all` 时有数据
    /// * `talkative_list` - 历史龙王，仅 `type` 为 `talkative` 或 `all` 时有数据
    /// * `performer_list` - 群聊之火，仅 `type` 为 `performer` 或 `all` 时有数据
    /// * `legend_list` - 群聊炽焰，仅 `type` 为 `legend` 或 `all` 时有数据
    /// * `strong_newbie_list` - 冒尖小春笋，仅 `type` 为 `strong_newbie` 或 `all` 时有数据
    /// * `emotion_list` - 快乐之源，仅 `type` 为 `emotion` 或 `all` 时有数据
    pub async fn get_group_honor_info(&self, group_id: i64, honor_type: &str) -> Result<Response> {
        let data = json!({"group_id": group_id, "type": honor_type});
        self.send_request("get_group_honor_info", data).await
    }

    // 凭证获取 API

    /// 获取 Cookies
    ///
    /// # 参数
    ///
    /// * `domain` - 需要获取 cookies 的域名
    ///
    /// # 响应数据
    ///
    /// * `cookies` - Cookies
    pub async fn get_cookies(&self, domain: &str) -> Result<Response> {
        let data = json!({"domain": domain});
        self.send_request("get_cookies", data).await
    }

    /// 获取 CSRF Token
    ///
    /// # 参数
    ///
    /// 无
    ///
    /// # 响应数据
    ///
    /// * `token` - CSRF Token (number int32)
    pub async fn get_csrf_token(&self) -> Result<Response> {
        self.send_request("get_csrf_token", json!({})).await
    }

    /// 获取 QQ 相关接口凭证
    ///
    /// 即上面两个接口的合并
    ///
    /// # 参数
    ///
    /// * `domain` - 需要获取 cookies 的域名
    ///
    /// # 响应数据
    ///
    /// * `cookies` - Cookies
    /// * `csrf_token` - CSRF Token (number int32)
    pub async fn get_credentials(&self, domain: &str) -> Result<Response> {
        let data = json!({"domain": domain});
        self.send_request("get_credentials", data).await
    }

    // 文件相关 API

    /// 获取语音
    ///
    /// # 参数
    ///
    /// * `file` - 收到的语音文件名（消息段的 `file` 参数），如 `0B38145AA44505000B38145AA4450500.silk`
    /// * `out_format` - 要转换到的格式，目前支持 `mp3`、`amr`、`wma`、`m4a`、`spx`、`ogg`、`wav`、`flac`
    ///
    /// # 说明
    ///
    /// 要使用此接口，通常需要安装 ffmpeg
    ///
    /// # 响应数据
    ///
    /// * `file` - 转换后的语音文件路径，如 `/home/somebody/cqhttp/data/record/0B38145AA44505000B38145AA4450500.mp3`
    pub async fn get_record(&self, file: &str, out_format: &str) -> Result<Response> {
        let data = json!({"file": file, "out_format": out_format});
        self.send_request("get_record", data).await
    }

    /// 获取图片
    ///
    /// # 参数
    ///
    /// * `file` - 收到的图片文件名（消息段的 `file` 参数），如 `6B4DE3DFD1BD271E3297859D41C530F5.jpg`
    ///
    /// # 响应数据
    ///
    /// * `file` - 下载后的图片文件路径，如 `/home/somebody/cqhttp/data/image/6B4DE3DFD1BD271E3297859D41C530F5.jpg`
    pub async fn get_image(&self, file: &str) -> Result<Response> {
        let data = json!({"file": file});
        self.send_request("get_image", data).await
    }

    // 功能检查 API

    /// 检查是否可以发送图片
    ///
    /// # 参数
    ///
    /// 无
    ///
    /// # 响应数据
    ///
    /// * `yes` - 是或否
    pub async fn can_send_image(&self) -> Result<Response> {
        self.send_request("can_send_image", json!({})).await
    }

    /// 检查是否可以发送语音
    ///
    /// # 参数
    ///
    /// 无
    ///
    /// # 响应数据
    ///
    /// * `yes` - 是或否
    pub async fn can_send_record(&self) -> Result<Response> {
        self.send_request("can_send_record", json!({})).await
    }

    // 状态和版本 API

    /// 获取运行状态
    ///
    /// # 参数
    ///
    /// 无
    ///
    /// # 响应数据
    ///
    /// * `online` - 当前 QQ 在线，`null` 表示无法查询到在线状态
    /// * `good` - 状态符合预期，意味着各模块正常运行、功能正常，且 QQ 在线
    /// * `……` - OneBot 实现自行添加的其它内容
    ///
    /// # 说明
    ///
    /// 通常情况下建议只使用 `online` 和 `good` 这两个字段来判断运行状态，
    /// 因为根据 OneBot 实现的不同，其它字段可能完全不同
    pub async fn get_status(&self) -> Result<Response> {
        self.send_request("get_status", json!({})).await
    }

    /// 获取版本信息
    ///
    /// # 参数
    ///
    /// 无
    ///
    /// # 响应数据
    ///
    /// * `app_name` - 应用标识，如 `mirai-native`
    /// * `app_version` - 应用版本，如 `1.2.3`
    /// * `protocol_version` - OneBot 标准版本，如 `v11`
    /// * `……` - OneBot 实现自行添加的其它内容
    pub async fn get_version_info(&self) -> Result<Response> {
        self.send_request("get_version_info", json!({})).await
    }

    // 系统操作 API

    /// 重启 OneBot 实现
    ///
    /// # 参数
    ///
    /// * `delay` - 要延迟的毫秒数，如果默认情况下无法重启，可以尝试设置延迟为 2000 左右，默认为 `0`
    ///
    /// # 说明
    ///
    /// 由于重启 OneBot 实现同时需要重启 API 服务，这意味着当前的 API 请求会被中断，
    /// 因此需要异步地重启，接口返回的 `status` 是 `async`
    ///
    /// # 响应数据
    ///
    /// 无
    pub async fn set_restart(&self, delay: i32) -> Result<Response> {
        let data = json!({"delay": delay});
        self.send_request("set_restart", data).await
    }

    /// 清理缓存
    ///
    /// 用于清理积攒了太多的缓存文件
    ///
    /// # 参数
    ///
    /// 无
    ///
    /// # 响应数据
    ///
    /// 无
    pub async fn clean_cache(&self) -> Result<Response> {
        self.send_request("clean_cache", json!({})).await
    }
}
