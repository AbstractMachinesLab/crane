ZAP_EXE=zap

.PHONY: build
build:
	cargo build

.PHONY: docs
docs:
	cargo doc --target-dir ./docs --workspace --no-deps

.PHONY: test
test:
	cargo test \
		--no-fail-fast \
		--tests

.PHONY: release.win
release.win: ZAP_EXE=zap.exe
release.win: release

.PHONY: release
release:
	cargo build --release
	tar czf release.tar.gz -C ./target/release/ $(ZAP_EXE)

.PHONY: install
install: release
	cargo install --path ./lib/zap-bin

.PHONY: setup
setup:

.PHONY: clean
clean:
	cargo clean

.PHONY: fmt
fmt:
	cargo fmt
