killall kanami
# LOG=debug nohup cargo run --release 2>&1 > log &
TZ=Asia/Shanghai nohup cargo run --release 2>&1 > log &
