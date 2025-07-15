#!/bin/bash

# 获取脚本所在目录和项目根目录
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# 编译项目
echo "正在编译 i18n-audit..."
cd "$PROJECT_ROOT"
cargo build --release || exit 1

# 运行示例程序
echo -e "\n--- 运行翻译示例程序 ---"
cd "$SCRIPT_DIR/mini_test"
cargo run || exit 1

# 运行审计工具
echo -e "\n--- 运行 i18n-audit ---"
cd "$SCRIPT_DIR"
"$PROJECT_ROOT/target/release/i18n-audit" -p "$SCRIPT_DIR/mini_test" --verbose

echo -e "\n--- 以 JSON 格式输出 ---"
"$PROJECT_ROOT/target/release/i18n-audit" -p "$SCRIPT_DIR/mini_test" run -f json

echo -e "\n--- 以 YAML 格式输出 ---"
"$PROJECT_ROOT/target/release/i18n-audit" -p "$SCRIPT_DIR/mini_test" run -f yaml

echo -e "\n--- 测试完成 ---"
# 返回到原始目录
cd "$PROJECT_ROOT" 