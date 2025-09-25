use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use dashmap::DashMap;
use lazy_static::lazy_static;
use rand::seq::IndexedRandom;

use crate::protocol::event::Event;

lazy_static! {
    static ref DB: DashMap<String, Vec<String>> = {
        let db = DashMap::new();
        db.entry("随机当归".to_string()).or_insert(
            "你的当归在摆烂/你的当归在摸鱼/你的当归在认真画画/你随到了在第五人格被追的吱哇乱叫的当归/你的当归压好一刀斩了/你的当归觉得你好闲，所以告诉你该写作业了/你随到了凹深渊的当归/你的当归说你出门时会下雨/你的当归新建了一个画布/你的当归新建了一个图层/你的当归一边看直播一边听歌一边新建图层/你打死这个当归她今天都不会再动一笔了！/你随到了女装当归/你的当归正在直播/你的当归口嗨说自己画完啦/你的当归在须弥迷路啦！/当归和弁天婚礼进行时…/当归正在煮牡蛎，不给你吃/当归提醒喝水小助手，多喝热水/当归被45℃的太阳烤干惹/当归啪给你一拳"
            .split('/')
            .map(|x| x.to_string())
            .collect()
        );
        db.entry("我想吃牡蛎".to_string()).or_insert(
            "你吃了一个蒜蓉粉丝蒸牡蛎/你吃了一个蒜香牡蛎/你吃了一个原味生蚝/你来了一碗牡蛎豆腐汤/你吃了一个捞汁儿小牡蛎/你吃了一个椒盐海牡蛎/你的牡蛎卧沙啦！/你吃了一个清蒸生蚝/你来了一碗牡蛎口蘑汤/你把牡蛎放进了麻辣烫加麻加辣/你的牡蛎哭了，哭的好大声/你的牡蛎被暴雨冲走了/你的牡蛎被当归吃了，你没得吃"
            .split('/')
            .map(|x| x.to_string())
            .collect()
        );
        db.entry("欺负一下牡蛎".to_string()).or_insert(
            "牡蛎被禁言1分钟/牡蛎被禁言3分钟/牡蛎被禁言5分钟/牡蛎被禁言10分钟/牡蛎被禁言30分钟/牡蛎被禁言1小时/牡蛎被禁言8小时/牡蛎被禁言10小时/牡蛎被禁言29天23小时59分/牡蛎的管理又不见啦！/牡蛎排队食堂两天没饭吃了，呜呜/牡蛎出门没带伞突然下暴雨/牡蛎抽卡大保底/牡蛎睡醒才想起昨晚作业ddl忘了交/牡蛎被六点喊起来做核酸"
            .split('/')
            .map(|x| x.to_string())
            .collect()
        );
        db.entry("中午吃啥".to_string()).or_insert(
            "你的外卖送丢了/你的外卖被人偷走啦！/清汤麻辣烫/番茄麻辣烫/麻辣烫加麻加辣/麻辣烫不麻不辣/麻辣烫，但是莴笋+1+1+1+1+1/牛肉麻辣香锅/鸡肉麻辣香锅/火锅冒菜/乡村基儿童套餐/肯德基疯狂星期四/麦当劳/汉堡王/干锅鱼豆腐/干锅千页豆腐/什锦炒饭/扬州炒饭/蛋包饭/咖喱鸡肉饭/照烧鸡肉饭/茭白肉丝盖饭/罐罐米线/土豆肉丝盖饭/青椒炒蛋盖饭/白粥/莎 县 小 吃/出门撸串吧/串串火锅！/过桥米线/渔粉/饵丝/别吃饭了吃当归吧/火锅！鸳鸯锅/火锅！番茄锅/火锅！九宫格/榴莲芝士披萨/焗饭/意大利面/酸辣粉/冰粉/凉糕/香菜新地/石锅拌饭/重庆鸡公煲/重庆小面/豌杂小面/牛肉小面/肥肠小面/牡蛎炖大鹅/螺蛳粉/抄手面/鲜虾净云吞/小笼包/华莱士/正新鸡排/手抓饼豪华版/手抓饼加鸡蛋/肉夹馍/老麻抄手/烤冷面/炸鸡可乐/炒冰淇淋/水果拼盘/蔬菜沙拉/铁板鱿鱼/冷面/凉面/凉皮/口水鸡/大盘鸡拌面/地三鲜/红烧茄子/宫保鸡丁/鱼香肉丝"
            .split('/')
            .map(|x| x.to_string())
            .collect()
        );
        db
    };
}

pub struct MuriApp;

#[async_trait]
impl super::Application for MuriApp {
    fn name(&self) -> &str {
        "template"
    }

    async fn on_event(&mut self, event: Arc<Event>) -> Result<()> {
        if let Event::MessageEvent(event) = event.as_ref() {
            let raw = event.raw_message();
            if let Some(key) = DB.get(raw) {
                let choise = key.choose(&mut rand::rng()).unwrap();
                event.reply(choise.to_string(), false).await?;
            }
        }
        Ok(())
    }
}

impl MuriApp {
    pub fn new() -> Self {
        Self {}
    }
}
