# Kanami

A suckless styled QQ bot client works with NapCat.

![](avatar.png)

## Motivation

py 虽然拿来写 bot 很舒服，但还是存在一些局限性：

1. 注解系统不够强大，比如异步函数忘调用 await 却没有提示，在代码 push 上去运行后才发现。
2. rs 的编译器检查可以你在写插件时排除掉绝大多数类似 1 的问题。
3. 对于错误处理，一个问号丢到上层比一堆 try except 舒服。
4. 我 vps 内存小，[anon](https://github.com/jerrita/anon)(py) 占用 40M，kanami(rs) 占用 5M
5. 这玩意只要编译通过了，很难爆炸。
6. 小蓝梁喜欢 rs。
7. 我导师喜欢 rs。
8. 我是变态，为了这点醋包的饺子。
9. Bang 15 便士。

## Design

不想写，你只要知道性能挺高就行。有兴趣看 `src/protocol/adapter.rs`。核心逻辑 100 行。

## Usage

1. fork 本项目
2. `cp src/config.def.rs src/config.rs` 并修改
3. 看 `src/application/template.rs`, 写你的应用逻辑
4. 改 `src/application/mod.rs` 的末尾，把你的应用加进去

```bash
# 如果 ENDPOINT 以 wss:// 打头
cargo run --release --features=tls
# 否则
cargo run --release
# 如果想改 log level
LOG=debug cargo run --release
```

## Evaluation

实验环境如下：
```
CPU: AMD EPYC 9274F 24-Core @ 48x 4.05GHz 
OS: Arch Linux x86_64 Linux 6.15.2-arch1-1
```

对从接收到消息开始，到插件处理完成的耗时大致如下：

![alt text](image.png)