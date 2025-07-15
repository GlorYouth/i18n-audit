use anyhow::{Result, Context};
use regex::Regex;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

use crate::{analyzer, config::Config, parser, report, scanner};

/// 动态调整翻译文件目录
pub fn adjust_locales_path(config: &mut Config) -> Result<()> {
    // 检查 rust_i18n::i18n!() 的调用并提取路径参数
    for entry in WalkDir::new(&config.src_path())
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
    {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            if let Ok(content) = fs::read_to_string(path) {
                let i18n_regex = Regex::new(r#"(?:rust_i18n::)?i18n!\s*\(\s*"([^"]+)"#).unwrap();
                if let Some(caps) = i18n_regex.captures(&content) {
                    if let Some(path_match) = caps.get(1) {
                        let i18n_path = path_match.as_str();
                        if config.verbose {
                            println!("检测到 i18n! 路径参数: {}", i18n_path);
                        }
                        config.locales_dir = i18n_path.to_string();
                        if config.verbose {
                            println!("调整后的翻译文件路径: {}", config.locales_dir);
                        }
                        return Ok(()); // 找到第一个就返回
                    }
                }
            }
        }
    }
    Ok(())
}

/// 运行审计命令
pub fn run_audit_command(config: &Config, format: &str, output: Option<&Path>) -> Result<()> {
    // 1. 扫描源代码
    let used_keys = scanner::scan_source_code(config).context("扫描源代码失败")?;

    // 2. 解析翻译文件
    let defined_keys = parser::parse_translation_files(config).context("解析翻译文件失败")?;

    // 3. 比对分析
    let analysis_result =
        analyzer::analyze(&used_keys, &defined_keys, config).context("分析翻译键使用情况失败")?;

    // 4. 生成报告
    let mut writer: Box<dyn std::io::Write> = if let Some(output_path) = output {
        Box::new(std::fs::File::create(output_path)?)
    } else {
        Box::new(std::io::stdout())
    };

    match format {
        "json" => report::print_json_report(&mut writer, &analysis_result, output)?,
        "yaml" => report::print_yaml_report(&mut writer, &analysis_result, output)?,
        _ => report::print_text_report(&mut writer, &analysis_result, config.threshold)?,
    }

    // 5. 根据阈值返回结果
    if analysis_result.unused_percentage > config.threshold {
        anyhow::bail!(
            "未使用的翻译键比例 ({:.2}%) 超过阈值 ({:.2}%)",
            analysis_result.unused_percentage,
            config.threshold
        );
    }

    Ok(())
}

/// 提取命令的入口点
pub fn run_extract_command(config: &Config) -> Result<()> {
    let used_keys = scanner::scan_source_code(config)
        .context("扫描源代码以提取键失败")?;
    
    let used_literal_keys: HashSet<String> = used_keys
        .iter()
        .filter(|k| k.is_literal)
        .map(|k| k.key.clone())
        .collect();

    if config.verbose {
        println!("找到 {} 个唯一的字面量使用键。", used_literal_keys.len());
    }

    extract_keys(config, used_literal_keys)?;

    println!("提取成功！翻译文件已按使用的键进行更新。");
    Ok(())
}

/// 提取逻辑的核心，可用于测试
pub fn extract_keys(config: &Config, used_literal_keys: HashSet<String>) -> Result<()> {
    let defined_keys = parser::parse_translation_files(config)
        .context("解析翻译文件以提取键失败")?;

    let mut keys_by_lang_and_file = HashMap::new();
    for key in defined_keys {
        keys_by_lang_and_file
            .entry(key.language.clone())
            .or_insert_with(HashMap::new)
            .entry(key.file_path.clone())
            .or_insert_with(BTreeMap::new)
            .insert(key.key, key.value);
    }
    
    for (lang, files) in keys_by_lang_and_file {
        if config.verbose {
            println!("正在处理语言: {}", lang);
        }
        for (file_path_str, defined_keys_map) in files {
            let absolute_path = config.project_path.join(&file_path_str);
            if config.verbose {
                println!("  正在处理文件: {}", absolute_path.display());
            }

            let filtered_keys: BTreeMap<String, String> = defined_keys_map
                .into_iter()
                .filter(|(key, _)| used_literal_keys.contains(key))
                .collect();

            if config.verbose {
                println!("    找到 {} 个使用中的键，准备写入文件。", filtered_keys.len());
            }
            
            parser::write_translation_file(&absolute_path, &filtered_keys)?;
        }
    }
    Ok(())
}

/// 格式化命令的入口点
pub fn run_format_command(config: &Config) -> Result<()> {
    let used_keys = scanner::scan_source_code(config)
        .context("扫描源代码以格式化失败")?;
    
    let master_key_set: BTreeMap<String, ()> = used_keys
        .iter()
        .filter(|k| k.is_literal)
        .map(|k| (k.key.clone(), ()))
        .collect();

    if config.verbose {
        println!("找到 {} 个唯一的字面量使用键作为格式化基准。", master_key_set.len());
    }

    format_keys(config, master_key_set)?;

    println!("格式化成功！翻译文件已排序和对齐。");
    Ok(())
}

/// 格式化逻辑的核心，可用于测试
pub fn format_keys(config: &Config, master_key_set: BTreeMap<String, ()>) -> Result<()> {
    let defined_keys = parser::parse_translation_files(config)
        .context("解析翻译文件以格式化失败")?;

    let mut keys_by_lang_and_file: HashMap<String, HashMap<String, HashMap<String, String>>> = HashMap::new();
    for key in defined_keys {
        keys_by_lang_and_file
            .entry(key.language.clone())
            .or_insert_with(HashMap::new)
            .entry(key.file_path.clone())
            .or_insert_with(HashMap::new)
            .insert(key.key, key.value);
    }
    
    for (lang, files) in keys_by_lang_and_file {
        if config.verbose {
            println!("正在格式化语言: {}", lang);
        }
        for (file_path_str, defined_keys_map) in files {
            let absolute_path = config.project_path.join(&file_path_str);
            if config.verbose {
                println!("  正在处理文件: {}", absolute_path.display());
            }

            let mut formatted_keys = BTreeMap::new();
            
            for (key, _) in &master_key_set {
                let value = defined_keys_map.get(key).cloned().unwrap_or_default();
                formatted_keys.insert(key.clone(), value);
            }
            
            if config.verbose {
                println!("    文件将以 {} 个键进行格式化。", formatted_keys.len());
            }
            
            parser::write_translation_file(&absolute_path, &formatted_keys)?;
        }
    }
    Ok(())
} 