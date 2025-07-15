use anyhow::{Result, Context};
use log::{info, debug};
use regex::Regex;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fs;
use walkdir::WalkDir;

use crate::config::Config;

/// 用于表示使用中的翻译键
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsedKey {
    /// 翻译键
    pub key: String,
    /// 是否是字面量键
    pub is_literal: bool,
    /// 所在文件路径
    pub file_path: String,
    /// 所在行号
    pub line_number: usize,
}

/// 扫描源代码，提取所有 t!() 宏调用中使用的键
pub fn scan_source_code(config: &Config) -> Result<Vec<UsedKey>> {
    let src_path = config.src_path();
    info!("正在扫描源代码目录: {}", src_path.display());
    
    let mut used_keys = Vec::new();
    let rust_file_extensions = [".rs"];
    
    for entry in WalkDir::new(&src_path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok()) 
    {
        let path = entry.path();
        
        // 只处理 Rust 文件
        if path.is_file() && path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| rust_file_extensions.contains(&ext))
            .unwrap_or(false)
        {
            debug!("处理文件: {}", path.display());
            
            // 读取文件内容
            let relative_path = path.strip_prefix(&config.project_path)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();
                
            let content = fs::read_to_string(path)
                .with_context(|| format!("无法读取文件: {}", path.display()))?;
                
            // 扫描文件内容中的 t!() 宏调用
            scan_file_content(&content, &relative_path, &mut used_keys)?;
        }
    }
    
    // 去重
    let mut unique_keys: HashMap<String, UsedKey> = HashMap::new();
    for key in used_keys {
        unique_keys.entry(key.key.clone())
            .or_insert_with(|| key);
    }
    
    let result: Vec<UsedKey> = unique_keys.into_values().collect();
    info!("扫描完成，找到 {} 个使用中的翻译键", result.len());
    
    Ok(result)
}

/// 扫描文件内容，提取所有 t!() 宏调用
fn scan_file_content(content: &str, file_path: &str, used_keys: &mut Vec<UsedKey>) -> Result<()> {
    // 使用正则表达式匹配 t!() 宏调用
    // 这里我们考虑以下几种形式：
    // 1. t!("literal.key")
    // 2. t!("literal.key", param = "value")
    // 3. t!(format!("dynamic.key.{}", var))
    // 4. t!(dynamic_key_var)
    
    // 匹配字面量键，包含以下模式:
    // - t!("key")
    // - t!("key", param = "value")
    // - t!("key", param1 = "value1", param2 = "value2")
    let literal_regex = Regex::new(r#"t!\s*\(\s*"([^"]+)"(?:\s*(?:,|\)))(?:[^)]*\))?"#)?;
    
    // 匹配动态键: t!(format!(...)) 或 t!(variable)
    let dynamic_regex = Regex::new(r#"t!\s*\(\s*(?!")[a-zA-Z0-9_]+(?:!|\b)"#)?;
    
    // 匹配format!内的模式
    let format_regex = Regex::new(r#"format!\s*\(\s*"([^"]+)"(?:\s*,)?"#)?;
    
    // 处理字面量键
    for (line_idx, line) in content.lines().enumerate() {
        for cap in literal_regex.captures_iter(line) {
            if let Some(key_match) = cap.get(1) {
                let key = key_match.as_str().to_string();
                debug!("在 {}:{} 找到字面量键: {}", file_path, line_idx + 1, key);
                
                used_keys.push(UsedKey {
                    key,
                    is_literal: true,
                    file_path: file_path.to_string(),
                    line_number: line_idx + 1,
                });
            }
        }
        
        // 处理动态键
        for cap in dynamic_regex.captures_iter(line) {
            if let Some(expr_match) = cap.get(0) {
                let expr_text = expr_match.as_str();
                debug!("在 {}:{} 发现可能的动态键表达式: {}", file_path, line_idx + 1, expr_text);
                
                // 查找附近的格式化字符串
                for format_cap in format_regex.captures_iter(line) {
                    if let Some(pattern_match) = format_cap.get(1) {
                        let pattern = pattern_match.as_str().to_string();
                        debug!("提取格式化模式: {}", pattern);
                        
                        // 尝试解析为动态键
                        used_keys.push(UsedKey {
                            key: pattern,
                            is_literal: false,
                            file_path: file_path.to_string(),
                            line_number: line_idx + 1,
                        });
                    }
                }
                
                // 检查变量声明
                let var_line_range = if line_idx >= 5 { line_idx - 5 } else { 0 }..=line_idx;
                let context_lines: Vec<&str> = content.lines().collect();
                
                for ctx_line_idx in var_line_range.rev().filter(|i| *i < context_lines.len()) {
                    let ctx_line = context_lines[ctx_line_idx];
                    
                    // 查找变量声明 let var = "key"; 或 let var = "prefix.key";
                    let var_regex = Regex::new(&format!(r#"let\s+([a-zA-Z0-9_]+)\s*=\s*"([^"]+)";"#)).unwrap();
                    
                    for var_cap in var_regex.captures_iter(ctx_line) {
                        if let (Some(var_name), Some(var_value)) = (var_cap.get(1), var_cap.get(2)) {
                            // 检查是否是我们正在寻找的变量
                            if line.contains(&format!("t!({})", var_name.as_str())) {
                                debug!("找到变量 {} = \"{}\" 在 {}:{}", 
                                      var_name.as_str(), var_value.as_str(), file_path, ctx_line_idx + 1);
                                
                                used_keys.push(UsedKey {
                                    key: var_value.as_str().to_string(),
                                    is_literal: false,
                                    file_path: file_path.to_string(),
                                    line_number: line_idx + 1,
                                });
                                
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(())
} 