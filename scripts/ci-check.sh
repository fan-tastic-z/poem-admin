#!/bin/bash

# CI 检查脚本 - 本地运行与 GitHub Actions 相同的检查
# 用法: ./scripts/ci-check.sh [--integration]

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 日志函数
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 检查必要的工具
check_prerequisites() {
    log_info "检查必要工具..."

    # 检查 Rust
    if ! command -v cargo &> /dev/null; then
        log_error "Cargo 未安装"
        exit 1
    fi

    # 检查 nextest
    if ! command -v cargo-nextest &> /dev/null; then
        log_warning "nextest 未安装，正在安装..."
        cargo install cargo-nextest
    fi

    # 检查 llvm-cov (如果需要覆盖率)
    if ! command -v cargo-llvm-cov &> /dev/null; then
        log_warning "llvm-cov 未安装，正在安装..."
        cargo install cargo-llvm-cov
    fi

    # 检查 Docker (如果运行集成测试)
    if [[ "$1" == "--integration" ]] && ! command -v docker &> /dev/null; then
        log_error "Docker 未安装，集成测试需要 Docker"
        exit 1
    fi

    log_success "工具检查完成"
}

# 代码格式检查
check_formatting() {
    log_info "检查代码格式..."
    if cargo fmt -- --check; then
        log_success "代码格式检查通过"
    else
        log_error "代码格式检查失败"
        log_info "运行 'cargo fmt' 来修复格式问题"
        exit 1
    fi
}

# Clippy 检查
check_lints() {
    log_info "运行 Clippy 检查..."
    if cargo clippy --all-targets --tests -- -D warnings; then
        log_success "Clippy 检查通过"
    else
        log_error "Clippy 检查失败"
        exit 1
    fi
}

# 构建检查
check_build() {
    log_info "检查项目构建..."
    if cargo check --all; then
        log_success "构建检查通过"
    else
        log_error "构建检查失败"
        exit 1
    fi
}

# 构建测试
build_tests() {
    log_info "构建测试..."
    if cargo build --tests; then
        log_success "测试构建完成"
    else
        log_error "测试构建失败"
        exit 1
    fi
}

# 单元测试
run_unit_tests() {
    log_info "运行单元测试..."
    if cargo nextest run --lib --bins; then
        log_success "单元测试通过"
    else
        log_error "单元测试失败"
        exit 1
    fi
}

# 集成测试
run_integration_tests() {
    log_info "运行集成测试..."

    # 设置测试环境变量
    export RUST_LOG=warn
    export TESTCONTAINERS_RYUK_DISABLED=true
    export TESTCONTAINERS_WAIT_TIMEOUT=60

    # 检查 Docker 状态
    if ! docker info &> /dev/null; then
        log_error "Docker 未运行，请启动 Docker"
        exit 1
    fi

    # 预拉取 PostgreSQL 镜像
    log_info "拉取 PostgreSQL Docker 镜像..."
    docker pull postgres:15

    # 运行业务逻辑集成测试
    log_info "运行业务逻辑集成测试..."
    if cargo nextest run --test integration_tests; then
        log_success "业务逻辑集成测试通过"
    else
        log_error "业务逻辑集成测试失败"
        exit 1
    fi

    # 运行 HTTP API 集成测试
    log_info "运行 HTTP API 集成测试..."
    if cargo nextest run --test api_integration_tests; then
        log_success "HTTP API 集成测试通过"
    else
        log_error "HTTP API 集成测试失败"
        exit 1
    fi

    # 运行所有集成测试
    log_info "运行完整集成测试套件..."
    if cargo nextest run --tests; then
        log_success "完整集成测试套件通过"
    else
        log_error "完整集成测试套件失败"
        exit 1
    fi
}

# 覆盖率报告
generate_coverage() {
    log_info "生成覆盖率报告..."
    if cargo llvm-cov nextest --html --lib --bins; then
        log_success "覆盖率报告生成完成"
        log_info "覆盖率报告: target/llvm-cov/html/index.html"
    else
        log_warning "覆盖率报告生成失败"
    fi
}

# 清理函数
cleanup() {
    log_info "清理临时文件..."
    # 这里可以添加清理逻辑
}

# 主函数
main() {
    local run_integration=false
    local generate_cov=false

    # 解析参数
    while [[ $# -gt 0 ]]; do
        case $1 in
            --integration)
                run_integration=true
                shift
                ;;
            --coverage)
                generate_cov=true
                shift
                ;;
            --help|-h)
                echo "用法: $0 [选项]"
                echo "选项:"
                echo "  --integration    运行集成测试"
                echo "  --coverage      生成覆盖率报告"
                echo "  --help, -h      显示此帮助信息"
                exit 0
                ;;
            *)
                log_error "未知选项: $1"
                exit 1
                ;;
        esac
    done

    # 设置陷阱来清理
    trap cleanup EXIT

    log_info "开始 CI 检查..."
    echo "========================================"

    # 基本检查
    check_prerequisites "$run_integration"
    check_formatting
    check_lints
    check_build
    build_tests
    run_unit_tests

    # 可选的覆盖率报告
    if [[ "$generate_cov" == true ]]; then
        generate_coverage
    fi

    # 集成测试
    if [[ "$run_integration" == true ]]; then
        echo "========================================"
        log_info "运行集成测试..."
        run_integration_tests
    fi

    echo "========================================"
    log_success "所有检查通过！🎉"

    if [[ "$run_integration" == false ]]; then
        log_info "提示: 使用 --integration 参数运行集成测试"
    fi
}

# 运行主函数
main "$@"
