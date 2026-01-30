VERSION := $(shell cargo metadata --no-deps --format-version=1 | jq -r '.packages[0].version')

.PHONY: all release build-debug build-release build-dll cp-release zip-release test clean installer

all:
	make build-debug
release:
	make clean
	make test
	make build-release
	make cp-release
	make zip-release
build-debug:
	cargo build --target x86_64-pc-windows-gnu --message-format=json | jq -r 'select(.executable) | .executable'
build-release:
	cargo build --target x86_64-pc-windows-gnu --message-format=json -r | jq -r 'select(.executable) | .executable'
build-dll:
	cargo build --target x86_64-pc-windows-gnu --lib --message-format=json -r | jq -r 'select(.filenames) | .filenames[]'
cp-release:
	rm -rf ./build
	mkdir -p ./build
	cp -rf target/x86_64-pc-windows-gnu/release/chamsae.exe ./build
	cp -rf target/x86_64-pc-windows-gnu/release/chamsae_settings.exe ./build
	cp -rf target/x86_64-pc-windows-gnu/release/chamsae.dll ./build
	cp -rf src/bat/*.bat ./build
	cd ./build && ./chamsae.exe -t
zip-release:
	cd ./build && zip -r ../chamsae-v$(VERSION).zip .
test:
	cargo test
clean:
	cargo clean
installer:
	@echo "InnoSetupでinstaller/chamsae.issをコンパイルしてください。"
	@echo "出力先: build/chamsae-v$(VERSION)-setup.exe"
