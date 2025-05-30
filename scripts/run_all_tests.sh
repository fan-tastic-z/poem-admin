#!/usr/bin/env bash
set -euo pipefail

# 运行所有集成测试的脚本
echo "🚀 Running all integration tests for poem-admin..."

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

# 设置环境变量
export RUST_LOG=${RUST_LOG:-"warn"}
export RUST_BACKTRACE=${RUST_BACKTRACE:-"1"}

echo ""
echo "📋 Test Plan:"
echo "1. Business Logic Integration Tests"
echo "2. HTTP API Integration Tests"
echo ""

# 运行业务逻辑集成测试
echo "🧪 Running Business Logic Integration Tests..."
echo "================================================"
if cargo test --test integration_tests -- --test-threads=1; then
    echo "✅ Business Logic Tests: PASSED"
else
    echo "❌ Business Logic Tests: FAILED"
    exit 1
fi

echo ""

# 运行HTTP API集成测试
echo "🌐 Running HTTP API Integration Tests..."
echo "========================================"
if cargo test --test api_integration_tests -- --test-threads=1; then
    echo "✅ HTTP API Tests: PASSED"
else
    echo "❌ HTTP API Tests: FAILED"
    exit 1
fi

echo ""
echo "🎉 All integration tests completed successfully!"
echo ""
echo "📊 Test Summary:"
echo "- Business Logic Tests: ✅ PASSED"
echo "- HTTP API Tests: ✅ PASSED"
echo ""
echo "🔍 For detailed test results, see TEST_RESULTS.md"
