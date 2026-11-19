.DEFAULT_GOAL := help

TEST_ENV ?= test_env
BIN      := rtree

.PHONY: help build run test check fmt lint clean install \
        demo demo-all demo-dirs demo-ascii demo-depth demo-nocolor demo-pipe \
        env-links env-reset ci

help: ## Show available commands
	@awk 'BEGIN {FS = ":.*##"}; /^[a-zA-Z_-]+:.*##/ {printf "%-14s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

# ---- build & quality ----------------------------------------------------

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

ci: fmt lint test ## Run everything before pushing
	@echo "all checks passed"

clean: ## Remove local build artifacts
	cargo clean

install: ## Install the release binary locally
	cargo install --path .

# ---- quick manual testing on $(TEST_ENV) --------------------------------

demo: ## Tree of test_env with defaults (colors, counts)
	cargo run -q -- $(TEST_ENV)

demo-all: ## Include hidden files and the summary footer
	cargo run -q -- -a -s $(TEST_ENV)

demo-dirs: ## Directories only
	cargo run -q -- -d $(TEST_ENV)

demo-ascii: ## ASCII connectors
	cargo run -q -- --ascii $(TEST_ENV)

demo-depth: ## Limit depth (make demo-depth D=2)
	cargo run -q -- -L $(or $(D),1) $(TEST_ENV)

demo-nocolor: ## Disable colors
	cargo run -q -- --no-color $(TEST_ENV)

demo-pipe: ## Piped output (colors should auto-disable)
	cargo run -q -- $(TEST_ENV) | cat

env-links: ## Create sample symlinks in test_env (file, dir, broken)
	@cd $(TEST_ENV) && \
	ln -sfn ../Makefile link-to-makefile && \
	ln -sfn . link-to-self && \
	ln -sfn does-not-exist broken-link
	@echo "created symlinks in $(TEST_ENV)"

env-reset: ## Remove the sample symlinks again
	@cd $(TEST_ENV) && rm -f link-to-makefile link-to-self broken-link
	@echo "removed symlinks from $(TEST_ENV)"
