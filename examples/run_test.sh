#!/bin/bash
set -e

# 默认格式为 text，可以从第一个参数覆盖
FORMAT=${1:-text}

# 获取脚本所在目录
SCRIPT_DIR=$(dirname "$(realpath "$0")")
PROJECT_ROOT=$(realpath "$SCRIPT_DIR/..")

# 编译项目
echo "--- 正在编译 i18n-audit... ---"
cargo build --release

# 查找所有 mini_test_* 目录
TEST_DIRS=$(find "$SCRIPT_DIR" -maxdepth 1 -type d -name "mini_test_*")

for test_dir in $TEST_DIRS; do
    test_name=$(basename "$test_dir")
    echo ""
    echo "--- 开始测试: $test_name ---"

    # 运行示例程序
    echo "--- 运行翻译示例程序 ($test_name) ---"
    cd "$test_dir"
    cargo run

    # 运行审计工具
    echo "--- 运行 i18n-audit (格式: $FORMAT, 测试: $test_name) ---"
    cd "$PROJECT_ROOT"
    ./target/release/i18n-audit -p "$test_dir" --verbose run -f "$FORMAT"
    
    echo "--- 测试完成: $test_name ---"
done

echo ""
echo "--- 所有测试已完成 ---" 