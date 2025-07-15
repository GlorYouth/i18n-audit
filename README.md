[For the Chinese version, please see `README_CN.md`](README_CN.md)

# I18n-Audit

A tool for auditing unused translation keys in rust-i18n projects.

## Features

- Scans Rust source code for `t!()` macro calls to extract all used translation keys.
- Parses YAML/JSON/TOML translation files to extract all defined translation keys.
- Compares the two to generate a report of unused translations.
- Supports analysis and warnings for dynamic keys.
- **Extracts in-use translation keys**, automatically cleaning up keys not used in the code.
- **Formats translation files**, aligning keys across all language files and providing placeholders for blank translations.
- Configurable warning threshold and ignore patterns.
- Supports multiple output formats: Text, JSON, YAML.
- Can be integrated into CI workflows.

## Installation

Note: This project is not currently planned for release on crates.io. Please use one of the following methods for installation:

**1. Install directly from GitHub via `cargo install` (Recommended):**
```bash
cargo install --git https://github.com/GlorYouth/i18n-audit
```

**2. Build from source:**
```bash
git clone https://github.com/GlorYouth/i18n-audit.git
cd i18n-audit
cargo build --release
```

## Usage

```bash
# Run an audit (run is the default command)
i18n-audit

# Extract the used translation keys and overwrite the translation files with them
# This will remove all keys not used in the code
i18n-audit extract

# Format all translation files to align their keys
# For missing translations, an empty string "" or '' will be used as a placeholder
i18n-audit format

# Specify the project path
i18n-audit -p /path/to/project

# Specify the source code and translation file directories
i18n-audit --src-dir src --locales-dir locales

# Generate a JSON format report and output to a file
i18n-audit run -f json -o report.json

# Set the warning threshold for unused translation keys (percentage)
i18n-audit --threshold 10

# Ignore keys matching a specific pattern (regular expression)
i18n-audit --ignore-pattern "^dynamic\\."

# By default, the tool ignores filenames starting with `TODO`
# Use this flag to include them
i18n-audit --no-ignore-todo

# Verbose mode
i18n-audit -v
```

## Output Example

Text format output:

```
I18n Translation Key Audit Report
┌────────────────┬──────────┐
│ Statistic Item │ Value    │
├────────────────┼──────────┤
│ Total Keys     │ 16       │
├────────────────┼──────────┤
│ Unused Keys    │ 8        │
├────────────────┼──────────┤
│ Missing Keys   │ 1        │
├────────────────┼──────────┤
│ Dynamic Keys   │ 1        │
├────────────────┼──────────┤
│ Unused Ratio   │ 50.00%   │
└────────────────┴──────────┘

Unused Translation Keys:
+----------+-------------------+-------------------+-------------------+
| Language | Key               | File Path         | Value             |
+==========+===================+===================+===================+
| en       | unused.key2       | locales\en.yml    | Unused Key 2      |
+----------+-------------------+-------------------+-------------------+
|          | unused.key1       | locales\en.yml    | Unused Key 1      |
+----------+-------------------+-------------------+-------------------+
|          | unused.nested.key | locales\en.yml    | Nested Unused Key |
+----------+-------------------+-------------------+-------------------+
|          | user.profile      | locales\en.yml    | User Profile      |
+----------+-------------------+-------------------+-------------------+
| zh-CN    | unused.key1       | locales\zh-CN.yml | 未使用的键1       |
+----------+-------------------+-------------------+-------------------+
|          | unused.key2       | locales\zh-CN.yml | 未使用的键2       |
+----------+-------------------+-------------------+-------------------+
|          | unused.nested.key | locales\zh-CN.yml | 嵌套的未使用键    |
+----------+-------------------+-------------------+-------------------+
|          | user.profile      | locales\zh-CN.yml | 用户资料          |
+----------+-------------------+-------------------+-------------------+

Missing Translation Keys:
+--------------------------+----------------+---------------------+
| Key                      | Location       | Missing Languages   |
+==========================+================+=====================+
| content.section.item.123 | src\main.rs:26 | en, zh-CN           |
+--------------------------+----------------+---------------------+

Dynamic Keys:
+-------------+----------------+
| Key Pattern | Location       |
+=============+================+
| dynamic.key | src\main.rs:19 |
+-------------+----------------+

Recommendation: The percentage of unused translation keys (50.00%) exceeds the threshold (20.00%). It is recommended to clean up unused translation keys.
```

## CI Integration

Example usage in a GitHub Actions workflow:

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

## Configuration

You can create an `.i18n-audit.toml` file in the project root for configuration:

```toml
# .i18n-audit.toml
src_dir = "src"
locales_dir = "locales"
threshold = 15.0
ignore_pattern = "^dynamic\\."
```

## License

This project is dual-licensed under either the [MIT License](LICENSE-MIT) or the [Apache License, Version 2.0](LICENSE-APACHE).

You may choose either license at your discretion. 