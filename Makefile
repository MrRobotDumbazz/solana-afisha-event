.DEFAULT_GOAL := help

DATABASE_URL ?= postgres://postgres:postgres@127.0.0.1:5432/afisha?sslmode=disable
RPC_URL ?= https://api.devnet.solana.com
PORT ?= 8080

help: ## Показать список целей
	@grep -E '^[a-zA-Z_-]+:.*?## ' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-16s\033[0m %s\n", $$1, $$2}'

build: build-program build-api build-web ## Собрать всё: anchor-программа + Go API + Vue

build-program: ## Собрать Solana-программу (anchor build)
	anchor build

build-api: ## Собрать Go API в services/api/bin/api
	cd services/api && go build -o bin/api .

build-web: ## Собрать Vue фронтенд (vite build)
	cd web && npm run build

test: test-program test-api ## Все тесты (litesvm + go)

test-program: ## Тесты Solana-программы (anchor test, litesvm)
	anchor test

test-api: ## Go-тесты
	cd services/api && go test ./...

dev: build-api ## Запустить API и фронт вместе (Ctrl+C останавливает оба)
	@cd services/api && DATABASE_URL="$(DATABASE_URL)" RPC_URL="$(RPC_URL)" PORT="$(PORT)" ./bin/api & \
	  api_pid=$$!; \
	  trap 'kill $$api_pid 2>/dev/null' INT TERM EXIT; \
	  cd web && npm run dev; \
	  kill $$api_pid 2>/dev/null

dev-api: build-api ## Запустить Go API (DATABASE_URL, RPC_URL из окружения)
	cd services/api && DATABASE_URL="$(DATABASE_URL)" RPC_URL="$(RPC_URL)" PORT="$(PORT)" ./bin/api

dev-web: ## Запустить vite dev-сервер на :5173
	cd web && npm run dev

deploy-devnet: build-program ## Задеплоить программу на devnet
	anchor deploy --provider.cluster devnet

fmt: ## Форматирование: rust + go
	cargo fmt -p events
	cd services/api && gofmt -w .

lint: ## Проверки: cargo clippy + go vet
	cargo clippy -p events --all-targets
	cd services/api && go vet ./...

install: ## Установить зависимости (npm, cargo fetch, go mod download)
	cd web && npm install
	cargo fetch
	cd services/api && go mod download

db-create: ## Создать локальную БД afisha (требуется postgres)
	psql "host=127.0.0.1 user=postgres password=postgres" -c "CREATE DATABASE afisha;"

db: ## Открыть psql для БД afisha
	psql "host=127.0.0.1 user=postgres password=postgres dbname=afisha"

clean: ## Удалить артефакты сборки (keypair программы в target/deploy сохраняется)
	rm -rf target/sbpf-solana-solana target/debug target/release target/idl target/types .anchor
	rm -f target/deploy/events.so
	rm -rf web/dist services/api/bin

.PHONY: help build build-program build-api build-web test test-program test-api dev dev-api dev-web deploy-devnet fmt lint install db-create db clean
