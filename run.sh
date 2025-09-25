killall kanami
# LOG=debug nohup cargo run --release 2>&1 > log &
LOG=info TZ=Asia/Shanghai nohup cargo run --release 2>&1 > log &
