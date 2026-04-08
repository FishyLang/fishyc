cargo build --release
cp ./target/release/fishyc ./fishyc
./fishyc test.fsh -O --dump-ir
cc fishy_app.o -o fishy_app
./fishy_app