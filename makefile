.PHONY: all build-debug build-release build-dll cp-release test clean

all:
	make build-debug
build-debug:
	cargo build --target x86_64-pc-windows-gnu --message-format=json | jq -r 'select(.executable) | .executable'
build-release:
	cargo build --target x86_64-pc-windows-gnu --message-format=json -r | jq -r 'select(.executable) | .executable'
	make cp-release
build-dll:
	cargo build --target x86_64-pc-windows-gnu --lib --message-format=json -r | jq -r 'select(.filenames) | .filenames[]'
cp-release:
	cp -rf target/x86_64-pc-windows-gnu/release/chamsae.exe .
	cp -rf target/x86_64-pc-windows-gnu/release/chamsae.dll .
test:
	cargo test
clean:
	cargo clean
