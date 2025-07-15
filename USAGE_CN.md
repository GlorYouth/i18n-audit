# i18n-audit 使用指南

## 概述

`i18n-audit` 是一个专为使用 `rust-i18n` 库的 Rust 项目设计的工具，它可以帮助您：

1.  **审计**：扫描源代码，找出未被使用的翻译键，以及代码中使用了但未在翻译文件中定义的键。
2.  **清理**：通过 `extract` 命令，自动移除翻译文件中未被使用的键。
3.  **格式化**：通过 `format` 命令，自动对齐所有语言的翻译文件，确保键的顺序一致，并为缺失的翻译添加占位符。

## 安装

```bash
cargo install i18n-audit
```

## 基本用法

### 命令行选项

```
USAGE:
    i18n-audit [OPTIONS] [COMMAND]

OPTIONS:
    -h, --help                           打印帮助信息
    -p, --path <PATH>                    项目根目录 [默认: .]
    --src-dir <SRC_DIR>                  源代码目录 [默认: src]
    --locales-dir <LOCALES_DIR>          翻译文件目录 [默认: locales]
    --threshold <THRESHOLD>              警告阈值百分比 [默认: 20.0]
    --ignore-pattern <IGNORE_PATTERN>    忽略匹配指定模式的键（正则表达式）
    --no-ignore-todo                     默认忽略翻译文件夹下 start_with TODO 的文件，此选项可关闭忽略
    -v, --verbose                        详细输出模式
    -V, --version                        打印版本信息

COMMANDS:
    run       运行审计并生成报告 (默认)
    extract   提取使用的翻译键并覆盖翻译文件
    format    格式化翻译文件，使键在各文件中对齐
    help      打印此帮助信息或给定子命令的帮助信息
```

### 命令详解

#### `run` (默认命令)

运行完整的审计流程，找出未使用和缺失的翻译键，并生成报告。

```bash
# 运行默认审计
i18n-audit

# 等同于
i18n-audit run

# 生成 JSON 格式报告并输出到文件
i18n-audit run -f json -o report.json
```

#### `extract`

扫描您的代码，找出所有实际使用到的翻译键，然后用这些键**覆盖**您现有的翻译文件。

这个命令非常适合用于**自动清理**那些在代码重构后遗留下来的、不再被使用的翻译键。

```bash
# 移除所有未使用的键
i18n-audit extract
```

#### `format`

扫描您的代码，找出所有实际使用到的翻译键，然后根据这份“主列表”来重写所有的翻译文件。

这个命令可以确保：
- 所有语言文件的键都以相同的字母顺序排列。
- 如果某个键在某一语言中缺失翻译，会自动为其添加一个空字符串 (`""` 或 `''`)作为占位符。

这使得发现和添加缺失的翻译变得非常容易。

```bash
# 对齐所有翻译文件中的键
i18n-audit format
```

### 其他选项示例

#### 生成不同格式的报告

```bash
# 文本格式（默认）
i18n-audit run -f text

# JSON 格式
i18n-audit run -f json

# YAML 格式
i18n-audit run -f yaml

# 输出到文件
i18n-audit run -f json -o report.json
```

#### 配置警告阈值和忽略模式

```bash
# 设置未使用翻译键的警告阈值（百分比）
i18n-audit --threshold 10

# 忽略特定模式的键（正则表达式）
i18n-audit --ignore-pattern "^temp\\."

# 组合使用
i18n-audit --threshold 15 --ignore-pattern "^(temp|test)\\."
```

## CI 集成

### GitHub Actions

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
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          profile: minimal
          toolchain: stable
      - name: Install i18n-audit
        run: cargo install i18n-audit
      - name: Run i18n-audit
        run: i18n-audit --threshold 15
```

### GitLab CI

```yaml
i18n-audit:
  stage: test
  image: rust:latest
  script:
    - cargo install i18n-audit
    - i18n-audit --threshold 15
  rules:
    - if: $CI_PIPELINE_SOURCE == 'merge_request_event'
    - if: $CI_COMMIT_BRANCH == $CI_DEFAULT_BRANCH
```

## 输出示例

### 文本格式

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

### JSON 格式

```json
{
  "unused_keys": {
    "en": [
      {
        "key": "common.unused.key1",
        "language": "en",
        "value": "Unused Key 1",
        "file_path": "locales/en.yml"
      },
      {
        "key": "common.unused.key2",
        "language": "en",
        "value": "Unused Key 2",
        "file_path": "locales/en.yml"
      }
    ],
    "zh-CN": [
      {
        "key": "common.unused.key1",
        "language": "zh-CN",
        "value": "未使用的键1",
        "file_path": "locales/zh-CN.yml"
      }
    ]
  },
  "missing_keys": [
    {
      "key": "common.button.submit",
      "missing_languages": ["zh-CN", "fr"],
      "file_path": "src/components/form.rs",
      "line_number": 15
    }
  ],
  "dynamic_keys": [
    {
      "pattern": "user.profile.{}",
      "file_path": "src/user/profile.rs",
      "line_number": 23
    }
  ],
  "stats": {
    "total_keys": 120,
    "total_unused": 25,
    "total_missing": 5,
    "total_dynamic": 3,
    "unused_percentage": 20.83
  }
}
```

## 最佳实践

1. **定期审计**：将 `i18n-audit run` 集成到您的 CI 流程中，定期检查翻译健康状况。
2. **清理无用键**：在大型重构后，使用 `i18n-audit extract` 来清理不再需要的键。
3. **同步翻译**：在添加新功能后，使用 `i18n-audit format` 来对齐所有语言文件，方便您快速定位和补充缺失的翻译。
4. **维护合理的阈值**：根据您的项目规模和翻译策略，设置合适的警告阈值。
5. **处理动态键**：对于动态键，使用 `--ignore-pattern` 选项忽略，或考虑重构为静态键。

## 注意事项

1. 该工具只能检测静态的、字面量的翻译键。对于动态生成的键，它会尝试进行分析并提供警告。

2. 如果您的项目使用了自定义的翻译宏或函数，而不是标准的 `t!()` 宏，该工具可能需要修改才能正常工作。

3. 当在大型项目上运行时，可能需要一些时间来扫描所有文件和分析结果。 