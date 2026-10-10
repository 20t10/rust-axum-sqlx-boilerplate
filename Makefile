.PHONY: setup dev up down reset-db migrate seed fmt lint test test-unit test-integration build-check check docs ci create-module init-project clean clean-doc target-size

ifneq (,$(strip $(MAKECMDGOALS)))
ifneq (,$(filter-out setup init-project fmt lint test-unit build-check docs create-module clean clean-doc target-size,$(MAKECMDGOALS)))
ifeq (,$(wildcard .env))
$(error missing required .env; run 'make setup')
endif
endif
endif

setup:
	cp -n .env.example .env || true

dev:
	cargo run -p api

up:
	docker compose up -d postgres

down:
	docker compose down

reset-db:
	docker compose down -v
	docker compose up -d postgres

migrate:
	@set -a; . ./.env; set +a; sqlx migrate run

seed:
	@set -a; . ./.env; set +a; psql "$$DATABASE_URL" -f seeds/dev.sql

fmt:
	cargo fmt --all

lint:
	cargo clippy --workspace --all-targets -- -D warnings

test:
	cargo test --workspace

test-unit:
	cargo test -p shared-kernel -p building-blocks -p identity

test-integration:
	cargo test --test identity_repository --test http_auth

build-check:
	cargo check --workspace --all-targets

clean:
	cargo clean

clean-doc:
	cargo clean --doc

target-size:
ifeq ($(OS),Windows_NT)
	powershell -NoProfile -File tools/target_size.ps1
else
	@if [ -d target ]; then du -sh target; else echo 'target does not exist'; fi
endif

check:
	cargo fmt --all --check
	cargo clippy --workspace --all-targets -- -D warnings
	cargo test --workspace

docs:
	@echo "OpenAPI endpoint: /docs"

ci:
	make check

create-module:
	./tools/create_module.sh $(name)

init-project:
	./tools/init_project.sh "$(name)"
