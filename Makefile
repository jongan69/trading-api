.PHONY: build run test test-live lint fmt clippy docker-build docker-run docs-gen clean

build:
	cargo build

run:
	cargo run

test:
	cargo test

test-live:
	cargo test -- --ignored

lint: fmt clippy
	cargo build

fmt:
	cargo fmt --check

clippy:
	cargo clippy --all-targets -- -D warnings

docker-build:
	docker build -t trading-api .

docker-run:
	docker run -p 3000:3000 --env-file .env trading-api

docs-gen:
	curl -s http://localhost:3000/openapi.json | python3 -m json.tool > openapi.json

clean:
	cargo clean
