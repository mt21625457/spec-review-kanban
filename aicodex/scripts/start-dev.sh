#!/bin/bash
# AICodex 多实例管理系统启动脚本
# 用法: ./start-dev.sh

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 项目根目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}   AICodex 多实例管理系统启动脚本${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# 检查依赖
check_dependencies() {
    echo -e "${YELLOW}检查依赖...${NC}"

    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}错误: 未找到 cargo，请安装 Rust${NC}"
        exit 1
    fi

    if ! command -v node &> /dev/null; then
        echo -e "${RED}错误: 未找到 node，请安装 Node.js${NC}"
        exit 1
    fi

    if ! command -v pnpm &> /dev/null; then
        echo -e "${YELLOW}警告: 未找到 pnpm，尝试使用 npm 安装...${NC}"
        npm install -g pnpm
    fi

    echo -e "${GREEN}依赖检查通过${NC}"
}

# 初始化环境
init_env() {
    echo -e "${YELLOW}初始化环境...${NC}"

    # 创建 .env 文件（如果不存在）
    if [ ! -f "$PROJECT_ROOT/aicodex/.env" ]; then
        cat > "$PROJECT_ROOT/aicodex/.env" << 'EOF'
# AICodex 开发环境配置
HOST=127.0.0.1
PORT=8765
DATABASE_URL=sqlite:./aicodex.db?mode=rwc
JWT_SECRET=dev-jwt-secret-please-change-in-production
CONFIG_ENCRYPTION_KEY=dev-encryption-key-32-bytes-long
VIBE_KANBAN_BIN=/Users/yangjianbo/code/mt-ai/spec-review-kanban/target/release/vibe-kanban
VIBE_INSTANCES_DATA_ROOT=./vibe-instances
VIBE_INSTANCES_PORT_BASE=18100
VIBE_INSTANCES_PORT_MAX=18199
RUST_LOG=aicodex=debug,tower_http=debug
EOF
        echo -e "${GREEN}已创建 aicodex/.env 文件${NC}"
    fi

    # 创建实例数据目录
    mkdir -p "$PROJECT_ROOT/aicodex/vibe-instances"
}

# 运行数据库迁移
run_migrations() {
    echo -e "${YELLOW}运行数据库迁移...${NC}"
    cd "$PROJECT_ROOT/aicodex"

    # 检查 sqlx-cli
    if ! command -v sqlx &> /dev/null; then
        echo -e "${YELLOW}安装 sqlx-cli...${NC}"
        cargo install sqlx-cli --no-default-features --features sqlite
    fi

    # 创建数据库并运行迁移
    sqlx database create 2>/dev/null || true
    sqlx migrate run

    echo -e "${GREEN}数据库迁移完成${NC}"
}

# 检查并创建管理员
create_admin_if_needed() {
    echo -e "${YELLOW}检查管理员用户...${NC}"
    cd "$PROJECT_ROOT/aicodex"

    # 检查数据库中是否有用户
    USER_COUNT=$(sqlite3 aicodex.db "SELECT COUNT(*) FROM users WHERE role='admin';" 2>/dev/null || echo "0")

    if [ "$USER_COUNT" = "0" ]; then
        echo -e "${YELLOW}未找到管理员用户，正在创建...${NC}"
        cargo run --bin create-admin -- \
            --username admin \
            --password admin123 \
            --display-name "系统管理员"
        echo -e "${GREEN}管理员用户已创建${NC}"
        echo -e "${YELLOW}  用户名: admin${NC}"
        echo -e "${YELLOW}  密码: admin123${NC}"
        echo -e "${RED}  请在生产环境中更改密码!${NC}"
    else
        echo -e "${GREEN}管理员用户已存在${NC}"
    fi
}

# 安装前端依赖
install_frontend_deps() {
    echo -e "${YELLOW}安装前端依赖...${NC}"
    cd "$PROJECT_ROOT/aicodex-web"

    if [ ! -d "node_modules" ]; then
        pnpm install
    fi

    echo -e "${GREEN}前端依赖安装完成${NC}"
}

# 构建 vibe-kanban（如果需要）
build_vibe_kanban() {
    VIBE_BINARY="$PROJECT_ROOT/target/release/vibe-kanban"

    if [ ! -f "$VIBE_BINARY" ]; then
        echo -e "${YELLOW}构建 vibe-kanban...${NC}"
        cd "$PROJECT_ROOT"
        cargo build --release -p vibe-kanban 2>/dev/null || cargo build --release
        echo -e "${GREEN}vibe-kanban 构建完成${NC}"
    else
        echo -e "${GREEN}vibe-kanban 已存在${NC}"
    fi
}

# 启动后端
start_backend() {
    echo -e "${YELLOW}启动后端服务...${NC}"
    cd "$PROJECT_ROOT/aicodex"
    cargo run &
    BACKEND_PID=$!
    echo -e "${GREEN}后端服务已启动 (PID: $BACKEND_PID)${NC}"
}

# 启动前端
start_frontend() {
    echo -e "${YELLOW}启动前端服务...${NC}"
    cd "$PROJECT_ROOT/aicodex-web"
    pnpm dev &
    FRONTEND_PID=$!
    echo -e "${GREEN}前端服务已启动 (PID: $FRONTEND_PID)${NC}"
}

# 清理函数
cleanup() {
    echo ""
    echo -e "${YELLOW}正在停止服务...${NC}"
    kill $BACKEND_PID 2>/dev/null || true
    kill $FRONTEND_PID 2>/dev/null || true
    echo -e "${GREEN}服务已停止${NC}"
    exit 0
}

# 主函数
main() {
    trap cleanup SIGINT SIGTERM

    check_dependencies
    init_env
    build_vibe_kanban
    run_migrations
    create_admin_if_needed
    install_frontend_deps

    echo ""
    echo -e "${BLUE}========================================${NC}"
    echo -e "${GREEN}启动服务...${NC}"
    echo -e "${BLUE}========================================${NC}"
    echo ""

    start_backend
    sleep 2
    start_frontend

    echo ""
    echo -e "${BLUE}========================================${NC}"
    echo -e "${GREEN}服务已启动!${NC}"
    echo -e "${BLUE}========================================${NC}"
    echo ""
    echo -e "  后端地址: ${GREEN}http://127.0.0.1:8765${NC}"
    echo -e "  前端地址: ${GREEN}http://localhost:5173${NC}"
    echo ""
    echo -e "  管理员账号: ${YELLOW}admin${NC}"
    echo -e "  管理员密码: ${YELLOW}admin123${NC}"
    echo ""
    echo -e "${YELLOW}按 Ctrl+C 停止服务${NC}"
    echo ""

    # 等待进程
    wait
}

main "$@"
