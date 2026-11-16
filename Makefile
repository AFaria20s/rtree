.DEFAULT_GOAL := help

.PHONY: help build run test check fmt lint clean install

help: ## Show available commands
	@awk 'BEGIN {FS = ":.*##"}; /^[a-zA-Z_-]+:.*##/ {printf "%-12s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

build: ## Build an optimized binary
	cargo build --release

run: ## Run rtree (pass ARGS='--help' for options)
	cargo run -- $(ARGS)

test: ## Run unit and integration tests
	cargo test

check: ## Type-check all targets
	cargo check --all-targets

fmt: ## Format the source tree
	cargo fmt --all

lint: ## Run Clippy with warnings denied
	cargo clippy --all-targets --all-features -- -D warnings

clean: ## Remove local build artifacts
	cargo clean

install: ## Install the release binary locally
	cargo install --path .
