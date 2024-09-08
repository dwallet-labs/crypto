setup:
	rustup update
	rustup component add clippy
	cargo install cargo-audit

build:
	cargo build

build-wasm:
	rustup target add wasm32-unknown-unknown
	cargo build --target wasm32-unknown-unknown

fmt:
	cargo fmt --all

test:
	cargo test --all-features --workspace

lint:
	cargo clippy --all-targets --all-features -- -D warnings

doc:
	cargo doc --workspace --no-deps --all-features --document-private-items --examples

bench:
	cargo bench --features benchmarking

audit:
	cargo audit