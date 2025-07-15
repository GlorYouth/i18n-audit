[For the Chinese version, please see `USAGE_CN.md`](USAGE_CN.md)

# i18n-audit Usage Guide

## Overview

`i18n-audit` is a tool designed for Rust projects using the `rust-i18n` library. It helps you to:

1.  **Audit**: Scan your source code to find unused translation keys and keys that are used in the code but not defined in the translation files.
2.  **Clean**: Automatically remove unused keys from your translation files with the `extract` command.
3.  **Format**: Automatically align the translation files for all languages, ensuring key order is consistent and adding placeholders for missing translations with the `format` command.

## Installation

```bash
cargo install i18n-audit
```

## Basic Usage

### Command-Line Options

```
USAGE:
    i18n-audit [OPTIONS] [COMMAND]

OPTIONS:
    -h, --help                           Print help information
    -p, --path <PATH>                    Project root directory [default: .]
    --src-dir <SRC_DIR>                  Source code directory [default: src]
    --locales-dir <LOCALES_DIR>          Translation file directory [default: locales]
    --threshold <THRESHOLD>              Warning threshold percentage [default: 20.0]
    --ignore-pattern <IGNORE_PATTERN>    Ignore keys matching a specific pattern (regular expression)
    --no-ignore-todo                     By default, files starting with `TODO` in the translation folder are ignored. This option disables that behavior.
    -v, --verbose                        Verbose mode
    -V, --version                        Print version information

COMMANDS:
    run       Run an audit and generate a report (default)
    extract   Extract used translation keys and overwrite translation files
    format    Format translation files to align keys
    help      Print this help message or the help of a given subcommand
```

### Command Details

#### `run` (Default Command)

Runs the full audit process, finds unused and missing translation keys, and generates a report.

```bash
# Run the default audit
i18n-audit

# Equivalent to
i18n-audit run

# Generate a JSON format report and output to a file
i18n-audit run -f json -o report.json
```

#### `extract`

Scans your code to find all actually used translation keys, then **overwrites** your existing translation files with only these keys.

This command is ideal for **automatically cleaning up** keys left over after code refactoring.

```bash
# Remove all unused keys
i18n-audit extract
```

#### `format`

Scans your code to find all actually used translation keys, then rewrites all translation files based on this "master list".

This command ensures that:
- Keys in all language files are sorted in the same alphabetical order.
- If a key is missing a translation in a certain language, an empty string (`""` or `''`) is automatically added as a placeholder.

This makes it very easy to find and add missing translations.

```bash
# Align keys in all translation files
i18n-audit format
```

### Other Option Examples

#### Generating Reports in Different Formats

```bash
# Text format (default)
i18n-audit run -f text

# JSON format
i18n-audit run -f json

# YAML format
i18n-audit run -f yaml

# Output to a file
i18n-audit run -f json -o report.json
```

#### Configuring Warning Threshold and Ignore Patterns

```bash
# Set the warning threshold for unused translation keys (percentage)
i18n-audit --threshold 10

# Ignore keys matching a specific pattern (regular expression)
i18n-audit --ignore-pattern "^temp\\."

# Combine options
i18n-audit --threshold 15 --ignore-pattern "^(temp|test)\\."
```

## CI Integration

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

## Output Example

### Text Format

```
I18n Translation Key Audit Report

Statistics:
  Total Keys: 120
  Unused Keys: 25
  Missing Keys: 5
  Dynamic Keys: 3
  Unused Percentage: 20.83%

Unused Keys:

  Language: en
    1. common.unused.key1 (locales/en.yml)
    2. common.unused.key2 (locales/en.yml)

  Language: zh-CN
    1. common.unused.key1 (locales/zh-CN.yml)
    2. common.unused.key2 (locales/zh-CN.yml)

Missing Keys:
  1. common.button.submit (src/components/form.rs:15)
     Missing Languages: zh-CN, fr

Dynamic Keys:
  1. user.profile.{} (src/user/profile.rs:23)

Recommendation:
  The percentage of unused translation keys (20.83%) exceeds the threshold (20.00%). It is recommended to clean up unused translation keys.
```

### JSON Format

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

## Best Practices

1. **Regular Audits**: Integrate `i18n-audit run` into your CI workflow to regularly check the health of your translations.
2. **Clean Up Unused Keys**: After major refactoring, use `i18n-audit extract` to clean up keys that are no longer needed.
3. **Synchronize Translations**: After adding new features, use `i18n-audit format` to align all language files, making it easy to locate and add missing translations.
4. **Maintain a Reasonable Threshold**: Set an appropriate warning threshold based on your project's size and translation strategy.
5. **Handle Dynamic Keys**: Use the `--ignore-pattern` option to ignore dynamic keys, or consider refactoring them to be static.

## Caveats

1. This tool can only detect static, literal translation keys. For dynamically generated keys, it will attempt to analyze them and provide a warning.

2. If your project uses custom translation macros or functions instead of the standard `t!()` macro, the tool may need modification to work correctly.

3. When run on large projects, it may take some time to scan all files and analyze the results. 