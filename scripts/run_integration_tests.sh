#!/bin/bash

# 集成测试运行脚本
# 用于本地开发和 CI 环境

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}🚀 Starting Integration Tests${NC}"

# 检查 Docker 是否可用
if ! command -v docker &> /dev/null; then
    echo -e "${RED}❌ Docker is not installed or not in PATH${NC}"
    exit 1
fi

if ! docker info &> /dev/null; then
    echo -e "${RED}❌ Docker daemon is not running${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Docker is available${NC}"

# 设置环境变量
export RUST_LOG=${RUST_LOG:-info}
export TESTCONTAINERS_RYUK_DISABLED=true
export TESTCONTAINERS_WAIT_TIMEOUT=120
export DOCKER_HOST=${DOCKER_HOST:-unix:///var/run/docker.sock}

# 拉取 PostgreSQL 镜像
echo -e "${YELLOW}📦 Pulling PostgreSQL Docker image...${NC}"
docker pull postgres:15

# 运行单元测试
echo -e "${YELLOW}🧪 Running unit tests...${NC}"
cargo nextest run --lib --bins

# 运行集成测试
echo -e "${YELLOW}🔧 Running integration tests...${NC}"
cargo nextest run --test integration_tests --nocapture

# API 集成测试状态信息
echo -e "${YELLOW}ℹ️  API integration tests are marked as ignored (require HTTP server setup)${NC}"
echo -e "${YELLOW}   They can be run manually with: cargo test --test api_integration_tests -- --ignored${NC}"

echo -e "${GREEN}✅ All integration tests completed successfully!${NC}"

# 清理 Docker 容器（可选）
if [ "${CLEANUP_DOCKER:-true}" = "true" ]; then
    echo -e "${YELLOW}🧹 Cleaning up Docker containers...${NC}"
    docker ps -aq --filter "ancestor=postgres:15" | xargs -r docker rm -f || true
fi

echo -e "${GREEN}🎉 Integration test run completed!${NC}"
