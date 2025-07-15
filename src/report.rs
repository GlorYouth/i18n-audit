use anyhow::{Context, Result};
use colored::*;
use serde_yaml;
use serde_json;
use std::io::Write;
use std::path::Path;
use prettytable::{row, Cell, Row, Table};

use crate::analyzer::{AnalysisResult, MissingKey, UnusedKey};

pub fn print_text_report(writer: &mut dyn Write, result: &AnalysisResult, threshold: f32) -> Result<()> {
    writeln!(writer, "\n{}", "I18n 翻译键审计报告".bold().underline())?;

    // 打印统计信息
    print_stats_table(writer, result)?;

    // 打印未使用的翻译键
    if !result.unused_keys.is_empty() {
        writeln!(
            writer,
            "\n{}",
            "未使用的翻译键:".yellow().bold()
        )?;
        print_unused_keys_table(writer, &result.unused_keys)?;
    }

    // 打印缺少翻译的键
    if !result.missing_keys.is_empty() {
        writeln!(
            writer,
            "\n{}",
            "缺少翻译的键:".red().bold()
        )?;
        print_missing_keys_table(writer, &result.missing_keys)?;
    }

    // 打印动态键
    if !result.dynamic_keys.is_empty() {
        writeln!(writer, "\n{}", "动态键:".cyan().bold())?;
        let mut table = Table::new();
        table.set_format(*prettytable::format::consts::FORMAT_DEFAULT);
        table.set_titles(row![b => "动态键模式", "位置"]);

        for key in &result.dynamic_keys {
            table.add_row(row![
                key.pattern,
                format!("{}:{}", key.file_path, key.line_number)
            ]);
        }
        table.print(writer)?;
    }

    // 打印建议
    if result.unused_percentage > threshold {
        writeln!(
            writer,
            "\n{}: {}",
            "建议".bold(),
            format!(
                "未使用的翻译键比例 ({:.2}%) 超过阈值 ({:.2}%)，建议清理未使用的翻译键。",
                result.unused_percentage, threshold
            )
            .yellow()
        )?;
    } else {
        writeln!(
            writer,
            "\n{}: {}",
            "建议".bold(),
            "干得不错！未使用的翻译键比例在阈值范围内。".green()
        )?;
    }

    Ok(())
}

fn print_stats_table(writer: &mut dyn Write, result: &AnalysisResult) -> Result<()> {
    let mut table = Table::new();
    table.set_format(*prettytable::format::consts::FORMAT_BOX_CHARS);
    
    table.set_titles(row![b -> "统计项目", b -> "值"]);
    table.add_row(row![
        "总翻译键数量", result.total_keys.to_string().green()
    ]);
    table.add_row(row![
        "未使用的翻译键", result.total_unused.to_string().yellow()
    ]);
    table.add_row(row![
        "缺少翻译的键", result.total_missing.to_string().red()
    ]);
    table.add_row(row![
        "动态键", result.total_dynamic.to_string().cyan()
    ]);
    table.add_row(row![
        "未使用比例", format!("{:.2}%", result.unused_percentage).yellow()
    ]);
    
    table.print(writer)?;
    Ok(())
}

fn print_unused_keys_table(
    writer: &mut dyn Write,
    unused_keys: &std::collections::HashMap<String, Vec<UnusedKey>>,
) -> Result<()> {
    let mut table = Table::new();
    table.set_format(*prettytable::format::consts::FORMAT_DEFAULT);
    table.set_titles(row![b => "语言", "翻译键", "文件路径", "值"]);

    for (language, keys) in unused_keys {
        for (i, key) in keys.iter().enumerate() {
            let lang_cell = if i == 0 {
                Cell::new(language).style_spec("b")
            } else {
                Cell::new("")
            };
            
            table.add_row(Row::new(vec![
                lang_cell,
                Cell::new(&key.key),
                Cell::new(&key.file_path),
                Cell::new(&key.value.chars().take(50).collect::<String>()),
            ]));
        }
    }
    
    table.print(writer)?;
    Ok(())
}

fn print_missing_keys_table(writer: &mut dyn Write, missing_keys: &[MissingKey]) -> Result<()> {
    let mut table = Table::new();
    table.set_format(*prettytable::format::consts::FORMAT_DEFAULT);
    table.set_titles(row![b => "翻译键", "位置", "缺少的语言"]);

    for key in missing_keys {
        table.add_row(row![
            key.key,
            format!("{}:{}", key.file_path, key.line_number),
            key.missing_languages.join(", ").red()
        ]);
    }
    
    table.print(writer)?;
    Ok(())
}

/// 将分析结果以 JSON 格式打印
pub fn print_json_report(writer: &mut dyn Write, result: &AnalysisResult, output_path: Option<&Path>) -> Result<()> {
    let json_str = serde_json::to_string_pretty(result)?;
    
    if let Some(path) = output_path {
        std::fs::write(path, json_str)
            .with_context(|| format!("无法将 JSON 报告写入文件: {}", path.display()))?;
    } else {
        writeln!(writer, "{}", json_str)?;
    }

    Ok(())
} 

/// 将分析结果以 YAML 格式打印
pub fn print_yaml_report(
    _writer: &mut dyn Write,
    result: &AnalysisResult,
    output_path: Option<&Path>,
) -> Result<()> {
    let yaml_str = serde_yaml::to_string(result)?;

    if let Some(path) = output_path {
        std::fs::write(path, yaml_str)
            .with_context(|| format!("无法将 YAML 报告写入文件: {}", path.display()))?;
    } else {
        println!("{}", yaml_str);
    }

    Ok(())
} 