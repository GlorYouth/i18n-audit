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
    // 5. rust_i18n::t!("literal.key")
    
    // 遍历每一行查找宏调用
    for (line_idx, line) in content.lines().enumerate() {
        // 处理命名空间的字面量键
        if line.contains("rust_i18n::t!") {
            // 处理带命名空间的字面量键: rust_i18n::t!("key") 
            let namespaced_regex = Regex::new(r#"rust_i18n::t!\s*\(\s*"([^"]+)"(?:\s*(?:,|\)))(?:[^)]*\))?"#)?;
            for cap in namespaced_regex.captures_iter(line) {
                if let Some(key_match) = cap.get(1) {
                    let key = key_match.as_str().to_string();
                    debug!("在 {}:{} 找到命名空间字面量键: {}", file_path, line_idx + 1, key);
                    
                    used_keys.push(UsedKey {
                        key,
                        is_literal: true,
                        file_path: file_path.to_string(),
                        line_number: line_idx + 1,
                    });
                }
            }

            // 处理命名空间动态键
            let ns_var_regex = Regex::new(r#"rust_i18n::t!\s*\(\s*([a-zA-Z0-9_]+)\s*\)"#)?;
            for cap in ns_var_regex.captures_iter(line) {
                if let Some(var_match) = cap.get(1) {
                    let var_name = var_match.as_str();
                    debug!("在 {}:{} 找到命名空间变量键引用: {}", file_path, line_idx + 1, var_name);
                    
                    process_dynamic_key(var_name, content, line_idx, file_path, used_keys)?;
                }
            }
        } else {
            // 处理标准字面量键: t!("key")
            let standard_regex = Regex::new(r#"t!\s*\(\s*"([^"]+)"(?:\s*(?:,|\)))(?:[^)]*\))?"#)?;
            for cap in standard_regex.captures_iter(line) {
                if let Some(key_match) = cap.get(1) {
                    let key = key_match.as_str().to_string();
                    debug!("在 {}:{} 找到标准字面量键: {}", file_path, line_idx + 1, key);
                    
                    used_keys.push(UsedKey {
                        key,
                        is_literal: true,
                        file_path: file_path.to_string(),
                        line_number: line_idx + 1,
                    });
                }
            }

            // 处理标准动态键
            let std_var_regex = Regex::new(r#"t!\s*\(\s*([a-zA-Z0-9_]+)\s*\)"#)?;
            for cap in std_var_regex.captures_iter(line) {
                if let Some(var_match) = cap.get(1) {
                    let var_name = var_match.as_str();
                    debug!("在 {}:{} 找到标准变量键引用: {}", file_path, line_idx + 1, var_name);
                    
                    process_dynamic_key(var_name, content, line_idx, file_path, used_keys)?;
                }
            }
        }
    }
    
    Ok(())
}

/// 处理动态键变量，查找变量定义并添加到使用键列表中
fn process_dynamic_key(
    var_name: &str,
    content: &str,
    line_idx: usize,
    file_path: &str,
    used_keys: &mut Vec<UsedKey>
) -> Result<()> {
    // 向上查找变量声明
    let context_start = if line_idx >= 20 { line_idx - 20 } else { 0 };
    let context_lines: Vec<&str> = content.lines().collect();
    let context = &context_lines[context_start..line_idx];
    
    // 查找变量声明 let var_name = "key";
    for ctx_line in context.iter().rev() {
        let var_decl_regex = Regex::new(&format!(r#"let\s+{}\s*=\s*"([^"]+)";"#, var_name))?;
        if let Some(decl_cap) = var_decl_regex.captures(ctx_line) {
            if let Some(key_value) = decl_cap.get(1) {
                debug!("  找到变量定义: {} = \"{}\"", var_name, key_value.as_str());
                
                used_keys.push(UsedKey {
                    key: key_value.as_str().to_string(),
                    is_literal: false,
                    file_path: file_path.to_string(),
                    line_number: line_idx + 1,
                });
                
                // 找到一个匹配就返回
                return Ok(());
            }
        }
    }
    
    // 没找到变量定义，添加默认键
    debug!("  未找到 {} 的变量定义", var_name);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_scan_file_content_standard_t_macro() {
        let content = r#"
        fn main() {
            println!("测试: {}", t!("greetings.hello"));
            println!("再见: {}", t!("greetings.goodbye"));
        }
        "#;
        let mut used_keys = Vec::new();
        
        scan_file_content(content, "test.rs", &mut used_keys).unwrap();
        
        assert_eq!(used_keys.len(), 2);
        assert_eq!(used_keys[0].key, "greetings.hello");
        assert_eq!(used_keys[1].key, "greetings.goodbye");
    }

    #[test]
    fn test_scan_file_content_namespaced_t_macro() {
        let content = r#"
        fn main() {
            println!("测试: {}", rust_i18n::t!("greetings.hello"));
            println!("欢迎: {}", rust_i18n::t!("user.welcome", name = "张三"));
        }
        "#;
        let mut used_keys = Vec::new();
        
        scan_file_content(content, "test.rs", &mut used_keys).unwrap();
        
        // 打印所有找到的键，帮助调试
        println!("找到的键:");
        for (i, key) in used_keys.iter().enumerate() {
            println!("  {}: {} ({})", i, key.key, key.file_path);
        }
        
        assert_eq!(used_keys.len(), 2, "应该只找到2个键，但找到了{}个", used_keys.len());
        
        // 检查是否包含预期的键
        let keys: Vec<String> = used_keys.iter().map(|k| k.key.clone()).collect();
        assert!(keys.contains(&"greetings.hello".to_string()), "未检测到 greetings.hello");
        assert!(keys.contains(&"user.welcome".to_string()), "未检测到 user.welcome");
    }

    #[test]
    fn test_scan_file_content_dynamic_key() {
        let content = r#"
        fn main() {
            let key = "dynamic.key";
            println!("动态键: {}", t!(key));
        }
        "#;
        let mut used_keys = Vec::new();
        
        scan_file_content(content, "test.rs", &mut used_keys).unwrap();
        
        assert_eq!(used_keys.len(), 1);
        assert_eq!(used_keys[0].key, "dynamic.key");
        assert_eq!(used_keys[0].is_literal, false);
    }

    #[test]
    fn test_scan_file_content_with_params() {
        let content = r#"
        fn main() {
            println!("欢迎: {}", t!("user.welcome", name = "张三"));
            println!("条目: {}", t!("content.section.item.123", count = 5));
        }
        "#;
        let mut used_keys = Vec::new();
        
        scan_file_content(content, "test.rs", &mut used_keys).unwrap();
        
        assert_eq!(used_keys.len(), 2);
        assert_eq!(used_keys[0].key, "user.welcome");
        assert_eq!(used_keys[1].key, "content.section.item.123");
    }

    #[test]
    fn test_scan_rust_i18n_example() {
        let content = r#"
        use rust_i18n::t;

        // 初始化翻译
        rust_i18n::i18n!("../locales");

        fn main() {
            // 设置当前语言
            rust_i18n::set_locale("zh-CN");
            
            // 示例 1: 使用字面量键
            println!("问候: {}", t!("greetings.hello"));
            println!("再见: {}", t!("greetings.goodbye"));
            
            // 示例 2: 带参数的翻译
            println!("欢迎信息: {}", t!("user.welcome", name = "张三"));
            
            // 示例 3: 动态键
            let key = "dynamic.key";
            println!("动态键: {}", t!(key));
            
            // 示例 4: 使用 format! 构建的动态键
            let section = "section";
            let id = 123;
            // 注意：rust-i18n 需要字面量键，因此这里只是示范如何分析这类情况
            let formatted_key = format!("content.{}.item.{}", section, id);
            println!("格式化动态键: {}", t!("content.section.item.123")); // 正确用法，使用字面量
            println!("(假设的动态键实现): {}", formatted_key); // 这里只是打印，不是实际调用 t!()
        }
        "#;
        
        let mut used_keys = Vec::new();
        scan_file_content(content, "mini_test.rs", &mut used_keys).unwrap();
        
        // 验证扫描结果
        assert!(used_keys.len() >= 4, "应当至少检测到4个翻译键，实际检测到: {}", used_keys.len());

        // 确保关键的翻译键被检测到
        let keys: Vec<String> = used_keys.iter().map(|k| k.key.clone()).collect();
        
        assert!(keys.contains(&"greetings.hello".to_string()), "未检测到 greetings.hello");
        assert!(keys.contains(&"greetings.goodbye".to_string()), "未检测到 greetings.goodbye");
        assert!(keys.contains(&"user.welcome".to_string()), "未检测到 user.welcome");
        assert!(keys.contains(&"content.section.item.123".to_string()), "未检测到 content.section.item.123");
        
        // 检查动态键是否被正确识别
        let dynamic_keys: Vec<&UsedKey> = used_keys.iter()
            .filter(|k| !k.is_literal)
            .collect();
        
        assert_eq!(dynamic_keys.len(), 1, "应当检测到1个动态键");
        assert_eq!(dynamic_keys[0].key, "dynamic.key");
    }
} 