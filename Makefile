.PHONY: build test cli api web check clean fmt lint

build:
	cargo build --workspace

test:
	cargo test --workspace

check:
	cargo clippy --workspace

fmt:
	cargo fmt

lint: fmt check

clean:
	cargo clean

cli:
	cargo run -p lod -- inspect examples/data.ttl

api:
	cargo run -p lod-api

web:
	cd apps/web && npm install && npm run dev

# Full CI pipeline (runs on push via .github/workflows/ci.yml)
ci: fmt lint build test