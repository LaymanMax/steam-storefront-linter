.PHONY: all build test lint format dev dev-server dev-web clean sample-db

all: build

build:
	pnpm --filter web build
	cargo build --release

test:
	cargo test --lib --workspace
	cargo test --tests --workspace
	pnpm --filter web test

lint:
	cargo clippy --workspace
	pnpm --filter web lint

format:
	cargo fmt --check

sample-db:
	cargo run -p linter-cli -- dataset sample

dev-server:
	cargo run -p linter-server

dev-web:
	pnpm --filter web dev

clean:
	cargo clean
	rm -rf apps/web/dist
