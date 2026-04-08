cargo build --release
cp ./target/release/fishyc ./fishyc
./fishyc test.fsh --dump-ir
cc fishy_app.o -o fishy_app
./fishy_app