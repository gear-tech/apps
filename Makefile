.PHONY: all check clean fmt fmt-check linter pre-check pre-commit prepare

all:
	@echo ──────────── Build release ────────────────────
	@cargo +nightly build --target wasm32-unknown-unknown --workspace --release
	@wasm-proc --path ./target/wasm32-unknown-unknown/release/*.wasm
	@ls -la ./target/wasm32-unknown-unknown/release/*.wasm

check: all
	@cargo +nightly test --workspace

clean:
	@echo ──────────── Clean ────────────────────────────
	@rm -rvf target

fmt:
	@echo ──────────── Format ───────────────────────────
	@cargo fmt --all

fmt-check:
	@echo ──────────── Check format ─────────────────────
	@cargo fmt --all -- --check

linter:
	@echo ──────────── Run linter ───────────────────────
	@cargo +nightly clippy --target wasm32-unknown-unknown --workspace -- --no-deps -D warnings -A "clippy::missing_safety_doc"

pre-check: fmt-check linter check

pre-commit: fmt linter check

prepare:
	@rustup toolchain add nightly
	@rustup target add wasm32-unknown-unknown --toolchain nightly
	@cargo install --locked --git https://github.com/gear-tech/gear wasm-proc
