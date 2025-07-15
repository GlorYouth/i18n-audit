use anyhow::Result;
use log::{info, warn};
use regex::Regex;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};

use crate::config::Config;
use crate::scanner::UsedKey;
use crate::parser::DefinedKey;

/// 翻译键分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    /// 按语言分组的未使用翻译键
    pub unused_keys: HashMap<String, Vec<UnusedKey>>,
    /// 缺少翻译的使用键
    pub missing_keys: Vec<MissingKey>,
    /// 动态键（可能需要特殊处理）
    pub dynamic_keys: Vec<DynamicKey>,
    /// 未使用翻译键的百分比
    pub unused_percentage: f32,
    /// 总翻译键数量
    pub total_keys: usize,
    /// 未使用的翻译键数量
    pub total_unused: usize,
    /// 缺少翻译的键数量
    pub total_missing: usize,
    /// 动态键数量
    pub total_dynamic: usize,
}

/// 未使用的翻译键
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnusedKey {
    /// 翻译键
    pub key: String,
    /// 语言代码
    pub language: String,
    /// 翻译值
    pub value: String,
    /// 所在文件路径
    pub file_path: String,
}

/// 缺少翻译的使用键
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissingKey {
    /// 翻译键
    pub key: String,
    /// 缺少翻译的语言列表
    pub missing_languages: Vec<String>,
    /// 所在文件路径
    pub file_path: String,
    /// 所在行号
    pub line_number: usize,
}

/// 动态键
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicKey {
    /// 键模式
    pub pattern: String,
    /// 所在文件路径
    pub file_path: String,
    /// 所在行号
    pub line_number: usize,
}

/// 分析使用键和定义键，生成分析结果
pub fn analyze(
    used_keys: &[UsedKey],
    defined_keys: &[DefinedKey],
    config: &Config,
) -> Result<AnalysisResult> {
    info!("正在分析翻译键使用情况");
    
    // 提取所有使用的字面量键
    let literal_used_keys: HashSet<String> = used_keys
        .iter()
        .filter(|k| k.is_literal)
        .map(|k| k.key.clone())
        .collect();
        
    // 提取动态键
    let dynamic_keys: Vec<DynamicKey> = used_keys
        .iter()
        .filter(|k| !k.is_literal)
        .map(|k| DynamicKey {
            pattern: k.key.clone(),
            file_path: k.file_path.clone(),
            line_number: k.line_number,
        })
        .collect();
    
    // 按语言分组的所有已定义键
    let mut defined_keys_by_language: HashMap<String, HashMap<String, DefinedKey>> = HashMap::new();
    
    // 所有已定义的语言集合
    let mut languages = HashSet::new();
    
    // 获取所有已定义键和语言
    for key in defined_keys {
        languages.insert(key.language.clone());
        
        defined_keys_by_language
            .entry(key.language.clone())
            .or_insert_with(HashMap::new)
            .insert(key.key.clone(), key.clone());
    }
    
    // 找出未使用的翻译键
    let mut unused_keys: HashMap<String, Vec<UnusedKey>> = HashMap::new();
    let mut total_unused = 0;
    
    // 应用忽略模式（如果有）
    let ignore_regex = if let Some(pattern) = &config.ignore_pattern {
        match Regex::new(pattern) {
            Ok(re) => Some(re),
            Err(err) => {
                warn!("忽略模式正则表达式无效: {}，错误: {}", pattern, err);
                None
            }
        }
    } else {
        None
    };
    
    for (language, keys) in &defined_keys_by_language {
        let mut unused_in_lang = Vec::new();
        
        for (key, def_key) in keys {
            // 检查是否应该忽略这个键
            let should_ignore = if let Some(re) = &ignore_regex {
                re.is_match(key)
            } else {
                false
            };
            
            if !should_ignore && !literal_used_keys.contains(key) {
                // 检查动态键模式是否匹配
                let mut matched_by_dynamic = false;
                
                // 检查是否与任何动态键模式匹配
                for dynamic_key in &dynamic_keys {
                    // 简单的前缀匹配
                    if key.starts_with(&dynamic_key.pattern) {
                        matched_by_dynamic = true;
                        break;
                    }
                    
                    // 检查是否包含占位符 {}
                    if dynamic_key.pattern.contains("{}") {
                        // 将占位符替换为正则表达式模式 (.+)
                        let pattern_regex_str = dynamic_key.pattern.replace("{}", "(.+)");
                        if let Ok(re) = Regex::new(&pattern_regex_str) {
                            if re.is_match(key) {
                                matched_by_dynamic = true;
                                break;
                            }
                        }
                    }
                }
                
                if !matched_by_dynamic {
                    unused_in_lang.push(UnusedKey {
                        key: key.clone(),
                        language: language.clone(),
                        value: def_key.value.clone(),
                        file_path: def_key.file_path.clone(),
                    });
                    total_unused += 1;
                }
            }
        }
        
        if !unused_in_lang.is_empty() {
            unused_keys.insert(language.clone(), unused_in_lang);
        }
    }
    
    // 找出缺少翻译的键
    let mut missing_keys = Vec::new();
    
    for used_key in used_keys.iter().filter(|k| k.is_literal) {
        let mut missing_languages = Vec::new();
        
        for language in &languages {
            if !defined_keys_by_language
                .get(language)
                .map(|keys| keys.contains_key(&used_key.key))
                .unwrap_or(false)
            {
                missing_languages.push(language.clone());
            }
        }
        
        if !missing_languages.is_empty() {
            missing_keys.push(MissingKey {
                key: used_key.key.clone(),
                missing_languages,
                file_path: used_key.file_path.clone(),
                line_number: used_key.line_number,
            });
        }
    }
    
    // 计算统计信息
    let total_keys = defined_keys.len();
    let unused_percentage = if total_keys > 0 {
        (total_unused as f32 / total_keys as f32) * 100.0
    } else {
        0.0
    };
    
    let result = AnalysisResult {
        unused_keys,
        missing_keys: missing_keys.clone(),
        dynamic_keys: dynamic_keys.clone(),
        unused_percentage,
        total_keys,
        total_unused,
        total_missing: missing_keys.len(),
        total_dynamic: dynamic_keys.len(),
    };
    
    info!("分析完成:");
    info!("  总翻译键数量: {}", result.total_keys);
    info!("  未使用的翻译键数量: {}", result.total_unused);
    info!("  缺少翻译的键数量: {}", result.total_missing);
    info!("  动态键数量: {}", result.total_dynamic);
    info!("  未使用翻译键百分比: {:.2}%", result.unused_percentage);
    
    Ok(result)
} 