.PHONY: all build-debug build-release test clean

all:
	make build-debug
build-debug:
	cargo build --target x86_64-pc-windows-gnu --message-format=json | jq -r 'select(.executable) | .executable'
build-release:
	cargo build --target x86_64-pc-windows-gnu --message-format=json -r | jq -r 'select(.executable) | .executable'
test:
	cargo test
clean:
	cargo clean
