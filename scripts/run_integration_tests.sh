#!/usr/bin/env bash
set -euo pipefail

# 集成测试执行脚本
echo "🚀 Starting integration tests for poem-admin..."

# 检查 Docker 是否运行
if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker is not running. Please start Docker first."
    exit 1
fi

# 设置Docker环境变量（支持OrbStack）
if [ -S "$HOME/.orbstack/run/docker.sock" ]; then
    export DOCKER_HOST="unix://$HOME/.orbstack/run/docker.sock"
    echo "🔧 Using OrbStack Docker socket"
fi

# 检查必要的工具
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo is not installed"
    exit 1
fi

if ! command -v cargo-nextest &> /dev/null; then
    echo "⚠️  cargo-nextest is not installed. Installing..."
    cargo install cargo-nextest --locked
fi

# 设置环境变量
export RUST_LOG=${RUST_LOG:-"info"}
export RUST_BACKTRACE=${RUST_BACKTRACE:-"1"}

# 清理旧的容器（如果存在）
echo "🧹 Cleaning up old test containers..."
docker ps -a --filter "name=test-postgres" --format "{{.ID}}" | xargs -r docker rm -f
docker network ls --filter "name=test-network" --format "{{.ID}}" | xargs -r docker network rm

# 创建测试网络
echo "🔧 Creating test network..."
docker network create test-network || true

# 编译项目
echo "🔨 Building project..."
cargo build --release

# 运行测试
echo "🧪 Running integration tests..."

# 运行业务逻辑集成测试（不需要 HTTP 服务器）
echo "📋 Running business logic integration tests..."
cargo nextest run --test integration_tests --profile default

# 运行 API 集成测试（使用真实的 HTTP 服务器）
echo "📡 Running HTTP API integration tests..."
cargo nextest run --test api_integration_tests --profile default

# 生成测试报告
echo "📊 Generating test reports..."
if [ -f "target/nextest/default/junit.xml" ]; then
    echo "✅ Test results saved to: target/nextest/default/junit.xml"
fi

# 清理测试网络
echo "🧹 Cleaning up test network..."
docker network rm test-network || true

echo "🎉 All integration tests completed successfully!"
echo ""
echo "Test Summary:"
echo "- ✅ Business Logic Tests: Passed"
echo "- ✅ HTTP API Tests: Passed"
echo ""
echo "For more details, check the test output above."

# 提供额外的测试选项
echo ""
echo "📝 Additional test commands:"
echo "  - Run quick tests: ./scripts/run_integration_tests.sh quick"
echo "  - Run with coverage: ./scripts/run_integration_tests.sh coverage"
echo "  - Run API tests: ./scripts/run_integration_tests.sh api"

# 处理命令行参数
case "${1:-default}" in
    "quick")
        echo "🏃 Running quick tests..."
        cargo nextest run --test integration_tests --profile quick
        ;;
    "coverage")
        echo "📈 Running tests with coverage..."
        if ! command -v cargo-llvm-cov &> /dev/null; then
            echo "Installing cargo-llvm-cov..."
            cargo install cargo-llvm-cov
        fi
        cargo llvm-cov nextest --test integration_tests --html --output-dir target/coverage
        echo "Coverage report generated: target/coverage/index.html"
        ;;
    "api")
        echo "🌐 Running API tests (including ignored)..."
        cargo nextest run --test api_integration_tests --run-ignored all
        ;;
    "all")
        echo "🔍 Running all tests..."
        cargo nextest run --all-tests --profile default
        ;;
    *)
        # Default case already handled above
        ;;
esac
