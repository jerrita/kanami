# AGENTS.md

Kanami：suckless 风格的 QQ bot 客户端，Rust 编写，对接 NapCat（OneBot 协议）。

## 硬规则

- 不要读取、打印或修改 `src/config.rs`：它是用户的私有配置（含 token、endpoint），已被 gitignore。需要知道配置项时看 `src/config.def.rs`。
- 新增配置项时只改 `src/config.def.rs`，并在汇报里说明用户需要同步到自己的 `config.rs`。
- 不要改 `.cargo/config.toml`。

## 常用命令

- 构建：`cargo build --release`
- 运行：`cargo run --release`；ENDPOINT 是 `wss://` 时加 `--features=tls`
- 日志级别：`LOG=debug cargo run --release`
- 部署：`run.sh`（`LOG=info TZ=Asia/Shanghai`，后台运行，输出到 `log`）

## 结构

- `src/main.rs`：初始化日志，启动协议适配器。
- `src/protocol/adapter.rs`：核心。WebSocket 连接、事件分发、重连，以及到应用层的桥接。
- `src/protocol/event/`、`src/protocol/message.rs`：OneBot 事件与消息类型。
- `src/protocol/extension/`：OneBot 与 NapCat 扩展 API（`napcat` feature，默认开启）。
- `src/application/mod.rs`：`Application` trait 与应用注册表。
- `src/application/*`：各个应用（builtin、chat、cat、cron、gscore、muri、ping）。新应用参照 `template.rs` 实现 `Application`，再到 `mod.rs` 末尾注册。

## 风格

- 保持 suckless：依赖少、代码短、不引入框架和多余抽象。
- 错误用 `anyhow` + `?` 往上抛。
- 提交信息用 Conventional Commits，scope 写应用或模块名，例如 `fix(gscore): daemon reconnect`、`feat(chat): reply with image`。
