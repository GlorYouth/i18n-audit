# I18n-Audit

一个用于审计 rust-i18n 项目中未使用的翻译键的工具。

## 特性

- 扫描 Rust 源代码中的 `t!()` 宏调用，提取所有使用的翻译键
- 解析 YAML/JSON/TOML 翻译文件，提取所有定义的翻译键
- 比对两者，生成未使用翻译的报告
- 支持动态键的分析和警告
- **提取使用中的翻译键**，自动清理未在代码中使用的键
- **格式化翻译文件**，在所有语言文件中对齐键，并为空白翻译提供占位符
- 可配置的警告阈值和忽略模式
- 支持多种输出格式：文本、JSON、YAML
- 可集成到 CI 流程中

## 安装

注意：本项目目前没有发布到 crates.io 的计划。请使用以下方法之一进行安装：

**1. 通过 `cargo install` 直接从 GitHub 安装 (推荐):**
```bash
cargo install --git https://github.com/GlorYouth/i18n-audit
```

**2. 从源码构建:**
```bash
git clone https://github.com/GlorYouth/i18n-audit.git
cd i18n-audit
cargo build --release
```

## 使用方法

```bash
# 运行审计 (run 是默认命令)
i18n-audit

# 提取使用中的翻译键，并用它们覆盖翻译文件
# 这会移除所有未在代码中使用的键
i18n-audit extract

# 格式化所有翻译文件，使其键在不同语言文件中对齐
# 对于缺失的翻译，会使用空字符串 "" 或 '' 作为占位符
i18n-audit format

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

# 默认情况下，工具会忽略以 `TODO` 开头的文件名
# 使用此标志来包含它们
i18n-audit --no-ignore-todo

# 详细模式
i18n-audit -v
```

## 输出示例

文本格式输出：

```
I18n 翻译键审计报告
┌────────────────┬──────────┐
│ 统计项目       │ 值       │
├────────────────┼──────────┤
│ 总翻译键数量   │ 16       │
├────────────────┼──────────┤
│ 未使用的翻译键 │ 8        │
├────────────────┼──────────┤
│ 缺少翻译的键   │ 1        │
├────────────────┼──────────┤
│ 动态键         │ 1        │
├────────────────┼──────────┤
│ 未使用比例     │ 50.00%   │
└────────────────┴──────────┘

未使用的翻译键:
+-------+-------------------+-------------------+-------------------+
| 语言  | 翻译键            | 文件路径          | 值                |
+=======+===================+===================+===================+
| en    | unused.key2       | locales\en.yml    | Unused Key 2      |
+-------+-------------------+-------------------+-------------------+
|       | unused.key1       | locales\en.yml    | Unused Key 1      |
+-------+-------------------+-------------------+-------------------+
|       | unused.nested.key | locales\en.yml    | Nested Unused Key |
+-------+-------------------+-------------------+-------------------+
|       | user.profile      | locales\en.yml    | User Profile      |
+-------+-------------------+-------------------+-------------------+
| zh-CN | unused.key1       | locales\zh-CN.yml | 未使用的键1       |
+-------+-------------------+-------------------+-------------------+
|       | unused.key2       | locales\zh-CN.yml | 未使用的键2       |
+-------+-------------------+-------------------+-------------------+
|       | unused.nested.key | locales\zh-CN.yml | 嵌套的未使用键    |
+-------+-------------------+-------------------+-------------------+
|       | user.profile      | locales\zh-CN.yml | 用户资料          |
+-------+-------------------+-------------------+-------------------+

缺少翻译的键:
+--------------------------+----------------+-------------+
| 翻译键                   | 位置           | 缺少的语言  |
+==========================+================+=============+
| content.section.item.123 | src\main.rs:26 | en, zh-CN   |
+--------------------------+----------------+-------------+

动态键:
+-------------+----------------+
| 动态键模式  | 位置           |
+=============+================+
| dynamic.key | src\main.rs:19 |
+-------------+----------------+

建议: 未使用的翻译键比例 (50.00%) 超过阈值 (20.00%)，建议清理未使用的翻译键。
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
        run: cargo install --git https://github.com/GlorYouth/i18n-audit
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

本项目采用双重许可： ([MIT 许可证](LICENSE-MIT) 或 [Apache 许可证 2.0 版](LICENSE-APACHE))。

您可以根据自己的偏好选择其中任意一种许可证。 