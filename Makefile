.PHONY: all check clean fmt fmt-check linter pre-check pre-commit prepare

all:
	@echo ──────────── Build release ────────────────────
	@cargo +nightly build --workspace --release
	@ls -la ./target/wasm32-unknown-unknown/release/*.wasm

check: all
	@cargo +nightly test --release --workspace
<<<<<<< HEAD
=======
	@cargo +nightly test --release --manifest-path ping/Cargo.toml
	@cargo +nightly test --release --manifest-path fungible-token/Cargo.toml
	@cargo +nightly test --release --manifest-path erc1155/Cargo.toml
	@cargo +nightly test --release --manifest-path nft-marketplace/marketplace/Cargo.toml
>>>>>>> update nft marketplace

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
	@cargo +nightly clippy --workspace -- --no-deps -D warnings -A "clippy::missing_safety_doc"

pre-check: fmt-check linter check

pre-commit: fmt linter check

prepare:
	@rustup toolchain add nightly
	@rustup target add wasm32-unknown-unknown --toolchain nightly
