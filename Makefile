.DEFAULT_GOAL := help

PORT ?= 3011
DEV_PORT := $(shell node scripts/find-free-port.mjs $(PORT) 2>/dev/null || echo $(PORT))
DEV_URL := http://127.0.0.1:$(DEV_PORT)

BLUE := \033[1;34m
CYAN := \033[1;36m
GREEN := \033[1;32m
YELLOW := \033[1;33m
DIM := \033[2m
RESET := \033[0m

.PHONY: help init dev desktop build build-desktop test upstream-status upstream-sync upstream-finalize

help: ## 显示可用的开发命令
	@printf "\n$(BLUE)Code: Bugrail · Development Commands$(RESET)\n"
	@printf "$(DIM)━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━$(RESET)\n\n"
	@printf "  $(GREEN)make init$(RESET)              安装锁定依赖\n"
	@printf "  $(GREEN)make dev$(RESET)               启动 Web 预览 · $(DEV_URL)\n"
	@printf "  $(CYAN)make desktop$(RESET)           启动 Tauri 桌面开发版\n"
	@printf "  $(CYAN)make test$(RESET)              运行前端测试\n"
	@printf "  $(CYAN)make build$(RESET)             构建静态前端\n"
	@printf "  $(CYAN)make build-desktop$(RESET)     构建本地 Tauri 桌面安装包\n"
	@printf "  $(YELLOW)make upstream-status$(RESET)   检查 CodeG 最新 release tag\n"
	@printf "  $(YELLOW)make upstream-sync$(RESET)     同步最新 CodeG release（可 TAG=vX.Y.Z）\n"
	@printf "  $(YELLOW)make upstream-finalize$(RESET) 标记某 release 为基线（需 TAG=vX.Y.Z）\n"
	@printf "\n$(DIM)使用 Ctrl+C 停止当前进程。首次运行请先执行 make init。$(RESET)\n\n"

init: ## 安装锁定依赖
	@pnpm install --frozen-lockfile

dev: ## 启动 Web 预览（自动选择空闲端口）
	@printf "\n$(GREEN)正在启动 Code: Bugrail 开发预览…$(RESET)\n"
	@printf "$(DIM)   页面地址  $(DEV_URL)$(RESET)\n"
	@printf "$(DIM)   端口 $(PORT) 被占用时自动 +1 探测；首次运行请先 make init。$(RESET)\n\n"
	@pnpm exec next dev --turbopack --hostname 127.0.0.1 --port $(DEV_PORT)

desktop: init ## 启动 Tauri 桌面开发版
	@pnpm tauri dev

test: ## 运行前端测试
	@pnpm test

build: ## 构建静态前端
	@pnpm build

build-desktop: init ## 构建本地 Tauri 桌面安装包
	@pnpm tauri build --debug --config '{"bundle":{"createUpdaterArtifacts":false}}'

upstream-status: ## 检查 CodeG 最新 release tag
	@pnpm upstream:status

UPSTREAM_TAG_ARGS := $(if $(TAG),--tag $(TAG),)

upstream-sync: ## 同步 CodeG release（默认最新，可指定 TAG=vX.Y.Z）
	@pnpm sync:upstream prepare $(UPSTREAM_TAG_ARGS)

upstream-finalize: ## 将某个 CodeG release 标记为基线（需指定 TAG=vX.Y.Z）
	@test -n "$(TAG)" || (echo "✗ 请指定 make upstream-finalize TAG=vX.Y.Z"; exit 1)
	@pnpm sync:upstream finalize --tag $(TAG)
