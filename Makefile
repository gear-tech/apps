.PHONY: all clean gtest fmt fmt-check linter pre-check pre-commit prepare

all:
	@echo ──────────── Build release ────────────────────
	@cargo +nightly build --target wasm32-unknown-unknown --workspace --release
	@wasm-proc --path ./target/wasm32-unknown-unknown/release/*.wasm
	@ls -la ./target/wasm32-unknown-unknown/release/*.wasm

clean:
	@echo ──────────── Clean ────────────────────────────
	@rm -rvf target

fmt:
	@echo ──────────── Format ───────────────────────────
	@cargo fmt --all

fmt-check:
	@echo ──────────── Check format ─────────────────────
	@cargo fmt --all -- --check

gtest: all
	cp ./target/wasm32-unknown-unknown/release/*.wasm ./gear/
	cd gear/gtest/src/js && npm i
	cd gear && gtest gtest/spec/test*.yaml

linter:
	@echo ──────────── Run linter ───────────────────────
	@cargo +nightly clippy --workspace -- --no-deps -D warnings -A "clippy::missing_safety_doc"

pre-check: fmt-check linter

pre-commit: fmt linter

prepare:
	@rustup toolchain add nightly
	@rustup target add wasm32-unknown-unknown --toolchain nightly
	@cargo install --git https://github.com/gear-tech/gear wasm-proc
	@cargo install --git https://github.com/gear-tech/gear gear-test
