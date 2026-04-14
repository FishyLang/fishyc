set -e

export LLVM_SYS_170_PREFIX=/usr/lib/llvm-17
export LLVM_CONFIG_PATH=/usr/bin/llvm-config-17

cargo build --release
cp ./target/release/fishyc ./fishyc
./fishyc test.fih --dump-ir --emit-llvm
cc fishy_app.o -o fishy_app
./fishy_app