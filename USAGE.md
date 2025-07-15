# i18n-audit 使用指南

## 概述

`i18n-audit` 是一个专为使用 `rust-i18n` 库的 Rust 项目设计的工具，它可以：

1. 扫描 Rust 源代码中所有 `t!()` 宏调用，提取使用的翻译键
2. 解析所有翻译文件，提取已定义的翻译键
3. 比对两者，生成未使用翻译的报告

## 安装

```bash
cargo install i18n-audit
```

## 基本用法

### 命令行选项

```
USAGE:
    i18n-audit [OPTIONS] [SUBCOMMAND]

OPTIONS:
    -p, --path <PATH>                    项目根目录，默认为当前目录 [default: .]
    --src-dir <SRC_DIR>                  源代码目录，默认为 src [default: src]
    --locales-dir <LOCALES_DIR>          翻译文件目录，默认为 locales [default: locales]
    --threshold <THRESHOLD>              警告阈值百分比 [default: 20.0]
    --ignore-pattern <IGNORE_PATTERN>    忽略匹配指定模式的键（正则表达式）
    -v, --verbose                        详细输出模式
    -h, --help                           打印帮助信息
    -V, --version                        打印版本信息

SUBCOMMANDS:
    run     运行审计并生成报告
    help    打印帮助信息
```

### 示例

#### 基本审计

```bash
# 在当前目录运行审计
i18n-audit

# 指定项目路径
i18n-audit -p /path/to/project

# 详细模式
i18n-audit -v
```

#### 自定义目录

```bash
# 指定源代码和翻译文件目录
i18n-audit --src-dir app/src --locales-dir resources/i18n
```

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

1. **定期审计**：将 i18n-audit 集成到您的 CI 流程中，定期检查未使用的翻译键

2. **维护合理的阈值**：根据您的项目规模和翻译策略，设置合适的警告阈值

3. **处理动态键**：对于动态键，使用 `--ignore-pattern` 选项忽略，或考虑重构为静态键

4. **添加到开发工作流**：在添加新功能或重构代码时，运行审计确保不会留下未使用的翻译

5. **清理未使用的键**：定期清理那些确认不再需要的未使用键，保持翻译文件的整洁

## 注意事项

1. 该工具只能检测静态的、字面量的翻译键。对于动态生成的键，它会尝试进行分析并提供警告。

2. 如果您的项目使用了自定义的翻译宏或函数，而不是标准的 `t!()` 宏，该工具可能需要修改才能正常工作。

3. 当在大型项目上运行时，可能需要一些时间来扫描所有文件和分析结果。 