set -e

cargo build --release
cp ./target/release/fishyc ./fishyc
./fishyc test.fsh --dump-ir --emit-llvm
cc fishy_app.o -o fishy_app
./fishy_app