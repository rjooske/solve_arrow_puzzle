#! zsh

cargo run --release --bin main &
MAIN_PID=$!
./scripts/mirror.sh
kill $MAIN_PID
