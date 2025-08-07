use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// OneBot 消息段枚举，支持所有标准消息段类型
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum Segment {
    /// 纯文本消息段
    Text {
        /// 纯文本内容
        text: String,
    },
    /// QQ 表情消息段
    Face {
        /// QQ 表情 ID
        id: String,
    },
    /// 图片消息段
    Image {
        /// 图片文件名、URL、绝对路径或 Base64 编码
        file: String,
        /// 图片类型，flash 表示闪照，无此参数表示普通图片
        #[serde(skip_serializing_if = "Option::is_none")]
        r#type: Option<String>,
        /// 图片 URL（接收时）
        #[serde(skip_serializing_if = "Option::is_none")]
        url: Option<String>,
        /// 是否使用已缓存的文件，默认 true
        #[serde(skip_serializing_if = "Option::is_none")]
        cache: Option<bool>,
        /// 是否通过代理下载文件，默认 true
        #[serde(skip_serializing_if = "Option::is_none")]
        proxy: Option<bool>,
        /// 下载网络文件的超时时间（秒）
        #[serde(skip_serializing_if = "Option::is_none")]
        timeout: Option<u32>,
    },
    /// 语音消息段
    Record {
        /// 语音文件名、URL、绝对路径或 Base64 编码
        file: String,
        /// 是否变声，默认 false
        #[serde(skip_serializing_if = "Option::is_none")]
        magic: Option<bool>,
        /// 语音 URL（接收时）
        #[serde(skip_serializing_if = "Option::is_none")]
        url: Option<String>,
        /// 是否使用已缓存的文件，默认 true
        #[serde(skip_serializing_if = "Option::is_none")]
        cache: Option<bool>,
        /// 是否通过代理下载文件，默认 true
        #[serde(skip_serializing_if = "Option::is_none")]
        proxy: Option<bool>,
        /// 下载网络文件的超时时间（秒）
        #[serde(skip_serializing_if = "Option::is_none")]
        timeout: Option<u32>,
    },
    /// 短视频消息段
    Video {
        /// 视频文件名、URL、绝对路径或 Base64 编码
        file: String,
        /// 视频 URL（接收时）
        #[serde(skip_serializing_if = "Option::is_none")]
        url: Option<String>,
        /// 是否使用已缓存的文件，默认 true
        #[serde(skip_serializing_if = "Option::is_none")]
        cache: Option<bool>,
        /// 是否通过代理下载文件，默认 true
        #[serde(skip_serializing_if = "Option::is_none")]
        proxy: Option<bool>,
        /// 下载网络文件的超时时间（秒）
        #[serde(skip_serializing_if = "Option::is_none")]
        timeout: Option<u32>,
    },
    /// @某人消息段
    At {
        /// @的 QQ 号，all 表示全体成员
        qq: String,
    },
    /// 猜拳魔法表情
    Rps,
    /// 掷骰子魔法表情
    Dice,
    /// 窗口抖动（戳一戳的简化形式）
    Shake,
    /// 戳一戳消息段
    Poke {
        /// 戳一戳类型
        r#type: String,
        /// 戳一戳 ID
        id: String,
        /// 表情名（接收时）
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },
    /// 匿名发消息（仅发送）
    Anonymous {
        /// 无法匿名时是否继续发送，默认 false
        #[serde(skip_serializing_if = "Option::is_none")]
        ignore: Option<bool>,
    },
    /// 链接分享消息段
    Share {
        /// 分享链接 URL
        url: String,
        /// 分享标题
        title: String,
        /// 内容描述
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<String>,
        /// 图片 URL
        #[serde(skip_serializing_if = "Option::is_none")]
        image: Option<String>,
    },
    /// 推荐联系人消息段（好友/群）
    Contact {
        /// 推荐类型：qq（好友）或 group（群）
        r#type: String,
        /// 被推荐的 QQ 号或群号
        id: String,
    },
    /// 位置消息段
    Location {
        /// 纬度
        lat: String,
        /// 经度
        lon: String,
        /// 位置标题
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        /// 位置内容描述
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<String>,
    },
    /// 音乐分享消息段
    Music {
        /// 音乐类型：qq、163、xm 或 custom
        r#type: String,
        /// 歌曲 ID（非自定义音乐）
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        /// 点击后跳转目标 URL（自定义音乐）
        #[serde(skip_serializing_if = "Option::is_none")]
        url: Option<String>,
        /// 音乐 URL（自定义音乐）
        #[serde(skip_serializing_if = "Option::is_none")]
        audio: Option<String>,
        /// 音乐标题（自定义音乐）
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        /// 内容描述（自定义音乐）
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<String>,
        /// 图片 URL（自定义音乐）
        #[serde(skip_serializing_if = "Option::is_none")]
        image: Option<String>,
    },
    /// 回复消息段
    Reply {
        /// 回复时引用的消息 ID
        id: String,
    },
    /// 合并转发消息段（仅接收）
    Forward {
        /// 合并转发 ID
        id: String,
    },
    /// 合并转发节点消息段
    Node {
        /// 转发的消息 ID（引用已有消息）
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        /// 发送者 QQ 号（自定义节点）
        #[serde(skip_serializing_if = "Option::is_none")]
        user_id: Option<String>,
        /// 发送者昵称（自定义节点）
        #[serde(skip_serializing_if = "Option::is_none")]
        nickname: Option<String>,
        /// 消息内容（自定义节点）
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<Value>,
    },
    /// XML 消息段
    Xml {
        /// XML 内容
        data: String,
    },
    /// JSON 消息段
    Json {
        /// JSON 内容
        data: String,
    },
}

/// OneBot 消息，由多个消息段组成的数组
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Message(pub Vec<Segment>);

impl fmt::Display for Segment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Segment::Text { text } => write!(f, "{}", text),
            Segment::Face { id } => write!(f, "[face:{}]", id),
            Segment::Image { file, .. } => write!(f, "[image:{}]", file),
            Segment::Record { file, .. } => write!(f, "[record:{}]", file),
            Segment::Video { file, .. } => write!(f, "[video:{}]", file),
            Segment::At { qq } => {
                if qq == "all" {
                    write!(f, "[@全体成员]")
                } else {
                    write!(f, "[@{}]", qq)
                }
            }
            Segment::Rps => write!(f, "[猜拳]"),
            Segment::Dice => write!(f, "[掷骰子]"),
            Segment::Shake => write!(f, "[戳一戳]"),
            Segment::Poke { name, .. } => {
                if let Some(name) = name {
                    write!(f, "[{}]", name)
                } else {
                    write!(f, "[戳一戳]")
                }
            }
            Segment::Anonymous { .. } => write!(f, "[匿名]"),
            Segment::Share { title, url, .. } => write!(f, "[分享:{} - {}]", title, url),
            Segment::Contact { r#type, id } => match r#type.as_str() {
                "qq" => write!(f, "[推荐好友:{}]", id),
                "group" => write!(f, "[推荐群:{}]", id),
                _ => write!(f, "[推荐联系人:{}:{}]", r#type, id),
            },
            Segment::Location {
                title, lat, lon, ..
            } => {
                if let Some(title) = title {
                    write!(f, "[位置:{} ({}, {})]", title, lat, lon)
                } else {
                    write!(f, "[位置:({}, {})]", lat, lon)
                }
            }
            Segment::Music { r#type, title, .. } => {
                if let Some(title) = title {
                    write!(f, "[音乐:{}]", title)
                } else {
                    write!(f, "[音乐分享:{}]", r#type)
                }
            }
            Segment::Reply { id } => write!(f, "[回复:{}]", id),
            Segment::Forward { id } => write!(f, "[合并转发:{}]", id),
            Segment::Node {
                user_id, nickname, ..
            } => {
                if let (Some(user_id), Some(nickname)) = (user_id, nickname) {
                    write!(f, "[转发节点:{}({})]", nickname, user_id)
                } else {
                    write!(f, "[转发节点]")
                }
            }
            Segment::Xml { .. } => write!(f, "[XML消息]"),
            Segment::Json { .. } => write!(f, "[JSON消息]"),
        }
    }
}
impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for segment in &self.0 {
            write!(f, "{}", segment)?;
        }
        Ok(())
    }
}

impl Message {
    /// 创建一个空消息
    pub fn new() -> Self {
        Message(Vec::new())
    }

    /// 添加一个消息段
    pub fn push(&mut self, segment: Segment) {
        self.0.push(segment);
    }

    /// 在指定位置插入一个消息段
    pub fn insert(&mut self, index: usize, segment: Segment) {
        self.0.insert(index, segment);
    }

    /// 获取消息段数组的引用
    pub fn segments(&self) -> &[Segment] {
        &self.0
    }

    /// 获取消息段数组的可变引用
    pub fn segments_mut(&mut self) -> &mut Vec<Segment> {
        &mut self.0
    }

    /// 判断消息是否为空
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// 获取消息段数量
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// 提取所有纯文本内容并合并
    pub fn plain_text(&self) -> String {
        self.0
            .iter()
            .filter_map(|segment| match segment {
                Segment::Text { text } => Some(text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("")
    }
}

impl Default for Message {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoIterator for Message {
    type Item = Segment;
    type IntoIter = std::vec::IntoIter<Segment>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Message {
    type Item = &'a Segment;
    type IntoIter = std::slice::Iter<'a, Segment>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a mut Message {
    type Item = &'a mut Segment;
    type IntoIter = std::slice::IterMut<'a, Segment>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl std::ops::Index<usize> for Message {
    type Output = Segment;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl std::ops::IndexMut<usize> for Message {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl From<Vec<Segment>> for Message {
    fn from(segments: Vec<Segment>) -> Self {
        Message(segments)
    }
}

impl From<Segment> for Message {
    fn from(segment: Segment) -> Self {
        Message(vec![segment])
    }
}

impl From<&str> for Message {
    fn from(text: &str) -> Self {
        Segment::Text {
            text: text.to_string(),
        }
        .into()
    }
}
impl From<String> for Message {
    fn from(text: String) -> Self {
        Segment::Text { text }.into()
    }
}
