# I18n-Audit

一个用于审计 rust-i18n 项目中未使用的翻译键的工具。

## 特性

- 扫描 Rust 源代码中的 `t!()` 宏调用，提取所有使用的翻译键
- 解析 YAML/JSON/TOML 翻译文件，提取所有定义的翻译键
- 比对两者，生成未使用翻译的报告
- 支持动态键的分析和警告
- 可配置的警告阈值和忽略模式
- 支持多种输出格式：文本、JSON、YAML
- 可集成到 CI 流程中

## 安装

```bash
cargo install i18n-audit
```

或者从源码构建：

```bash
git clone https://github.com/yourusername/i18n-audit.git
cd i18n-audit
cargo build --release
```

## 使用方法

基本用法：

```bash
# 在当前目录运行审计
i18n-audit

# 指定项目路径
i18n-audit -p /path/to/project

# 指定源代码和翻译文件目录
i18n-audit --src-dir src --locales-dir locales

# 生成 JSON 格式报告并输出到文件
i18n-audit run -f json -o report.json

# 设置未使用翻译键的警告阈值（百分比）
i18n-audit --threshold 10

# 忽略特定模式的键（正则表达式）
i18n-audit --ignore-pattern "^dynamic\\."

# 详细模式
i18n-audit -v
```

## 输出示例

文本格式输出：

```
I18n 翻译键审计报告

统计信息:
  总翻译键数量: 120
  未使用的翻译键数量: 25
  缺少翻译的键数量: 5
  动态键数量: 3
  未使用翻译键百分比: 20.83%

未使用的翻译键:

  语言: en
    1. common.unused.key1 (locales/en.yml)
    2. common.unused.key2 (locales/en.yml)

  语言: zh-CN
    1. common.unused.key1 (locales/zh-CN.yml)
    2. common.unused.key2 (locales/zh-CN.yml)

缺少翻译的键:
  1. common.button.submit (src/components/form.rs:15)
     缺少语言: zh-CN, fr

动态键:
  1. user.profile.{} (src/user/profile.rs:23)

建议:
  未使用的翻译键比例 (20.83%) 超过阈值 (20.00%)，建议清理未使用的翻译键。
```

## CI 集成

在 GitHub Actions 工作流中使用示例：

```yaml
name: I18n Audit

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  i18n-audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install i18n-audit
        run: cargo install i18n-audit
      - name: Run i18n-audit
        run: i18n-audit --threshold 15
```

## 配置

可以在项目根目录创建 `.i18n-audit.toml` 文件进行配置：

```toml
# .i18n-audit.toml
src_dir = "src"
locales_dir = "locales"
threshold = 15.0
ignore_pattern = "^dynamic\\."
```

## 许可证

MIT 