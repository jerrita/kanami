# AGENTS.md

Kanami：suckless 风格的 QQ bot 客户端，Rust 编写，对接 NapCat（OneBot 协议）。

## 硬规则

- 不要读取或打印 `config.toml`（以及 `KANAMI_CONFIG` 指向的配置文件）：它是用户的私有配置，含 token、endpoint。
- 系统配置项以 `src/config.rs` 为准，应用配置项以各应用模块的配置结构体为准；新增或修改配置项时同步更新 `config.example.toml`。
- 只有逻辑通用、部署者确有必要修改的项才暴露为配置；应用自带数据和实现细节保留在应用代码中（如 muri 词库、cat API 地址及图片数量上限、chat 摘要文案和发送间隔）。
- 配置解析错误保留字段路径，不打印配置文件原文。
- `[alias.group]` / `[alias.user]` 是独立命名空间。应用配置中的群号/QQ 号字段用 `i64`，通过 `config::group_id` / `config::user_id` 反序列化；系统配置加载完成后再解析应用配置。不要恢复 `main_group` 专用配置或 target。
- 不要改 `.cargo/config.toml`。

## 常用命令

- 构建：`cargo build --release`
- 运行：`cargo run --release`；`endpoint` 是 `wss://` 时加 `--features=tls`
- 日志级别：`LOG=debug cargo run --release`
- 部署：`run.sh`（`LOG=info TZ=Asia/Shanghai`，后台运行，输出到 `log`）

## 结构

- `src/main.rs`：初始化日志，启动协议适配器。
- `src/protocol/adapter.rs`：核心。WebSocket 连接、事件分发、重连，以及到应用层的桥接。
- `src/protocol/event/`、`src/protocol/message.rs`：OneBot 事件与消息类型。新增事件时同步维护 `Event::group_id()` / `user_id()`，框架名单覆盖所有带对应 ID 的事件。
- `src/protocol/extension/`：OneBot 与 NapCat 扩展 API（`napcat` feature，默认开启）。
- `src/application/mod.rs`：`Application` trait 与应用注册表。先完成所有应用的 `new`、`start`，再连接 WS；重连不得重新调用应用生命周期方法。
- `src/application/*`：各个应用（builtin、chat、cat、cron、gscore、muri、ping）。新应用参照 `template.rs` 实现 `Application`，在自身模块定义配置结构体，再到 `mod.rs` 注册表增加一行 `mount::<YourApp>()`。顶层 `apps` 列表控制启用和顺序，配置放在 `[app.<name>]`；`config.rs` 不引用应用类型。可选 `groups` / `users` allow/deny 名单由挂载 wrapper 处理，应用内不要重复过滤。

## 风格

- 保持 suckless：依赖少、代码短、不引入框架和多余抽象。
- 错误用 `anyhow` + `?` 往上抛。
- 提交信息用 Conventional Commits，scope 写应用或模块名，例如 `fix(gscore): daemon reconnect`、`feat(chat): reply with image`。
