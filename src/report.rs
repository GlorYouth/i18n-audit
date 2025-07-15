use anyhow::{Result, Context};
use colored::*;
use log::info;
use serde::{Serialize, Deserialize};
use serde_yaml;
use serde_json;
use std::fs;
use std::io::{self, Write};
use std::path::Path;

use crate::analyzer::AnalysisResult;
use crate::config::Config;

/// 序列化的分析结果，用于输出为 JSON/YAML
#[derive(Serialize, Deserialize, Debug)]
struct SerializableResult {
    /// 未使用的翻译键，按语言分组
    unused_keys: serde_json::Value,
    /// 缺少翻译的使用键
    missing_keys: serde_json::Value,
    /// 动态键
    dynamic_keys: serde_json::Value,
    /// 统计信息
    stats: Stats,
}

/// 统计信息
#[derive(Serialize, Deserialize, Debug)]
struct Stats {
    /// 总翻译键数量
    total_keys: usize,
    /// 未使用的翻译键数量
    total_unused: usize,
    /// 缺少翻译的键数量
    total_missing: usize,
    /// 动态键数量
    total_dynamic: usize,
    /// 未使用翻译键的百分比
    unused_percentage: f32,
}

/// 生成报告
pub fn generate_report(
    result: &AnalysisResult,
    format: &str,
    output_path: Option<&Path>,
    config: &Config,
) -> Result<()> {
    info!("正在生成报告，格式: {}", format);
    
    match format {
        "text" => generate_text_report(result, output_path, config)?,
        "json" => generate_json_report(result, output_path, config)?,
        "yaml" => generate_yaml_report(result, output_path, config)?,
        _ => {
            anyhow::bail!("不支持的输出格式: {}，支持的格式: text, json, yaml", format);
        }
    }
    
    Ok(())
}

/// 生成文本格式的报告
fn generate_text_report(
    result: &AnalysisResult,
    output_path: Option<&Path>,
    config: &Config,
) -> Result<()> {
    let mut output = Vec::new();
    
    writeln!(&mut output, "\n{}", "I18n 翻译键审计报告".bold().underline())?;
    writeln!(&mut output, "\n{}", "统计信息:".bold())?;
    writeln!(&mut output, "  总翻译键数量: {}", result.total_keys)?;
    writeln!(&mut output, "  未使用的翻译键数量: {}", result.total_unused)?;
    writeln!(&mut output, "  缺少翻译的键数量: {}", result.total_missing)?;
    writeln!(&mut output, "  动态键数量: {}", result.total_dynamic)?;
    writeln!(&mut output, "  未使用翻译键百分比: {:.2}%", result.unused_percentage)?;
    
    // 未使用的键（按语言分组）
    if !result.unused_keys.is_empty() {
        writeln!(&mut output, "\n{}", "未使用的翻译键:".bold().yellow())?;
        
        for (language, keys) in &result.unused_keys {
            writeln!(&mut output, "\n  语言: {}", language.bold())?;
            
            for (idx, unused_key) in keys.iter().enumerate() {
                writeln!(&mut output, "    {}. {} ({})", 
                    idx + 1, 
                    unused_key.key.yellow(), 
                    unused_key.file_path
                )?;
                
                if config.verbose {
                    writeln!(&mut output, "       值: \"{}\"", unused_key.value)?;
                }
            }
        }
    }
    
    // 缺少翻译的键
    if !result.missing_keys.is_empty() {
        writeln!(&mut output, "\n{}", "缺少翻译的键:".bold().red())?;
        
        for (idx, missing_key) in result.missing_keys.iter().enumerate() {
            writeln!(&mut output, "  {}. {} ({}:{})", 
                idx + 1,
                missing_key.key.red(),
                missing_key.file_path,
                missing_key.line_number
            )?;
            
            writeln!(&mut output, "     缺少语言: {}", 
                missing_key.missing_languages.join(", ")
            )?;
        }
    }
    
    // 动态键
    if !result.dynamic_keys.is_empty() {
        writeln!(&mut output, "\n{}", "动态键:".bold().blue())?;
        
        for (idx, dynamic_key) in result.dynamic_keys.iter().enumerate() {
            writeln!(&mut output, "  {}. {} ({}:{})", 
                idx + 1,
                dynamic_key.pattern.blue(),
                dynamic_key.file_path,
                dynamic_key.line_number
            )?;
        }
    }
    
    // 添加删除建议
    if result.unused_percentage > config.threshold {
        writeln!(&mut output, "\n{}", "建议:".bold())?;
        writeln!(&mut output, "  未使用的翻译键比例 ({:.2}%) 超过阈值 ({:.2}%)，建议清理未使用的翻译键。", 
            result.unused_percentage, 
            config.threshold
        )?;
    }
    
    // 输出报告
    if let Some(path) = output_path {
        fs::write(path, output)
            .with_context(|| format!("无法写入报告文件: {}", path.display()))?;
        
        info!("报告已写入文件: {}", path.display());
    } else {
        io::stdout().write_all(&output)?;
    }
    
    Ok(())
}

/// 生成 JSON 格式的报告
fn generate_json_report(
    result: &AnalysisResult,
    output_path: Option<&Path>,
    _config: &Config,
) -> Result<()> {
    // 将分析结果转换为可序列化的结构
    let serializable = SerializableResult {
        unused_keys: serde_json::to_value(&result.unused_keys)?,
        missing_keys: serde_json::to_value(&result.missing_keys)?,
        dynamic_keys: serde_json::to_value(&result.dynamic_keys)?,
        stats: Stats {
            total_keys: result.total_keys,
            total_unused: result.total_unused,
            total_missing: result.total_missing,
            total_dynamic: result.total_dynamic,
            unused_percentage: result.unused_percentage,
        },
    };
    
    let json = serde_json::to_string_pretty(&serializable)?;
    
    // 输出报告
    if let Some(path) = output_path {
        fs::write(path, json)
            .with_context(|| format!("无法写入报告文件: {}", path.display()))?;
        
        info!("JSON 报告已写入文件: {}", path.display());
    } else {
        println!("{}", json);
    }
    
    Ok(())
}

/// 生成 YAML 格式的报告
fn generate_yaml_report(
    result: &AnalysisResult,
    output_path: Option<&Path>,
    _config: &Config,
) -> Result<()> {
    // 将分析结果转换为可序列化的结构
    let serializable = SerializableResult {
        unused_keys: serde_json::to_value(&result.unused_keys)?,
        missing_keys: serde_json::to_value(&result.missing_keys)?,
        dynamic_keys: serde_json::to_value(&result.dynamic_keys)?,
        stats: Stats {
            total_keys: result.total_keys,
            total_unused: result.total_unused,
            total_missing: result.total_missing,
            total_dynamic: result.total_dynamic,
            unused_percentage: result.unused_percentage,
        },
    };
    
    let yaml = serde_yaml::to_string(&serializable)?;
    
    // 输出报告
    if let Some(path) = output_path {
        fs::write(path, yaml)
            .with_context(|| format!("无法写入报告文件: {}", path.display()))?;
        
        info!("YAML 报告已写入文件: {}", path.display());
    } else {
        println!("{}", yaml);
    }
    
    Ok(())
} 