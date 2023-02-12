.DEFAULT_GOAL := help

ifeq ($(OS),Windows_NT)
    detected_OS := Windows
else
    detected_OS := $(shell uname)
endif

.PHONY: help
help:
ifeq ($(detected_OS), Windows)
	@grep -E "^[a-zA-Z_/-]+:.*?## .*$$" $(MAKEFILE_LIST) | sort | awk "BEGIN {FS = \":.*?## \"}; { printf \"%%-30s %%s\r\n\", $$1, $$2 }"
else
	@grep -E '^[a-zA-Z_/-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'
endif

.PHONY: docker-compose/build
docker-compose/build: ## Build or rebuild services
	docker-compose build

.PHONY: docker-compose/down
docker-compose/down: ## Stop and remove containers, networks
	docker-compose down

.PHONY: docker-compose/up
docker-compose/up: docker-compose/build ## Create and start containers
	docker-compose up -d

.PHONY: docker-compose/up-postgres
docker-compose/up-postgres: ## Run postgres
	docker-compose up -d postgres

.PHONY: rust/clean
rust/clean: ## Remove the target directory
	cargo clean

.PHONY: rust/clippy
rust/clippy: ## Checks a package to catch common mistakes and improve your Rust code.
	cargo clippy

.PHONY: rust/deps
rust/deps: ## Install dependencies
	rustup target add wasm32-unknown-unknown
	cargo install --locked sqlx-cli trunk

.PHONY: rust/fmt
rust/fmt: ## This utility formats all bin and lib files of the current crate using rustfmt.
	cargo fmt

.PHONY: rust/migrate
rust/migrate: ## Migrate database
	sqlx database setup --source ./crates/chameleon-backend/migrations/

.PHONY: rust/prepare
rust/prepare: ## Generate query metadata to support offline compile-time verification.
	cargo sqlx prepare --merged

.PHONY: rust/run-backend
rust/run-backend: ## Run backend
	cargo run --bin chameleon-backend

.PHONY: rust/run-frontend
rust/run-frontend: ## Run frontend
	trunk serve

.PHONY: clean
clean: docker-compose/down rust/clean ## Clean

.PHONY: format
format: rust/fmt ## Format

.PHONY: lint
lint: rust/clippy ## Lint

.PHONY: run/backend
run-backend: rust/deps docker-compose/up-postgres rust/migrate rust/run-backend ## Run backend

.PHONY: run/frontend
run-frontend: rust/deps rust/run-frontend ## Run frontend
