use i18n_audit::{config, scanner, parser, analyzer, report};

use clap::{Parser, Subcommand};
use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use regex::Regex;
use std::fs;
use std::collections::{BTreeMap, HashMap, HashSet};

/// i18n-audit - 用于审计 rust-i18n 项目中未使用的翻译键
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// 项目根目录，默认为当前目录
    #[arg(short, long, default_value = ".")]
    path: PathBuf,
    
    /// 源代码目录，默认为 src
    #[arg(long, default_value = "src")]
    src_dir: String,
    
    /// 翻译文件目录，默认为 locales
    #[arg(long, default_value = "locales")]
    locales_dir: String,
    
    /// 警告阈值百分比，当未使用翻译键超过此百分比时发出警告
    #[arg(long, default_value_t = 20.0)]
    threshold: f32,
    
    /// 忽略匹配指定模式的键（正则表达式）
    #[arg(long)]
    ignore_pattern: Option<String>,

    /// 不要忽略以 "TODO" 开头的文件
    #[arg(long)]
    no_ignore_todo: bool,
    
    /// 详细输出模式
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// 运行审计并生成报告
    Run {
        /// 输出格式: text, json, yaml
        #[arg(short, long, default_value = "text")]
        format: String,
        
        /// 输出文件路径，如未指定则输出到控制台
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// 提取使用的翻译键并覆盖翻译文件
    Extract,
    /// 格式化翻译文件，使键在各文件中对齐
    Format,
}

fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();
    
    // 创建配置
    let mut config = config::Config {
        project_path: cli.path,
        src_dir: cli.src_dir,
        locales_dir: cli.locales_dir.clone(),
        threshold: cli.threshold,
        ignore_pattern: cli.ignore_pattern,
        ignore_todo_files: !cli.no_ignore_todo,
        verbose: cli.verbose,
    };
    
    // 动态调整翻译文件目录
    adjust_locales_path(&mut config)?;

    match cli.command.unwrap_or(Commands::Run { 
        format: "text".to_string(), 
        output: None 
    }) {
        Commands::Run { format, output } => {
            run_audit(&config, &format, output.as_deref())?
        }
        Commands::Extract => {
            run_extract(&config)?
        }
        Commands::Format => {
            run_format(&config)?
        }
    }

    Ok(())
}

/// 动态调整翻译文件目录
fn adjust_locales_path(config: &mut config::Config) -> Result<()> {
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

/// 运行审计
fn run_audit(config: &config::Config, format: &str, output: Option<&Path>) -> Result<()> {
    // 1. 扫描源代码
    let used_keys = scanner::scan_source_code(config)
        .context("扫描源代码失败")?;
    
    // 2. 解析翻译文件
    let defined_keys = parser::parse_translation_files(config)
        .context("解析翻译文件失败")?;
            
    // 3. 比对分析
    let analysis_result = analyzer::analyze(&used_keys, &defined_keys, config)
        .context("分析翻译键使用情况失败")?;
    
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

/// 提取使用的翻译键
fn run_extract(config: &config::Config) -> Result<()> {
    // 1. 扫描源代码获取 used_keys
    let used_keys = scanner::scan_source_code(config)
        .context("扫描源代码以提取键失败")?;
    
    // 我们只关心字面量键，因为动态键无法精确确定
    let used_literal_keys: HashSet<String> = used_keys
        .iter()
        .filter(|k| k.is_literal)
        .map(|k| k.key.clone())
        .collect();

    if config.verbose {
        println!("找到 {} 个唯一的字面量使用键。", used_literal_keys.len());
    }

    // 2. 解析翻译文件获取 defined_keys
    let defined_keys = parser::parse_translation_files(config)
        .context("解析翻译文件以提取键失败")?;

    // 3. 按语言和文件路径对已定义的键进行分组
    // HashMap<language, HashMap<file_path, BTreeMap<key, value>>>
    let mut keys_by_lang_and_file = HashMap::new();
    for key in defined_keys {
        keys_by_lang_and_file
            .entry(key.language.clone())
            .or_insert_with(HashMap::new)
            .entry(key.file_path.clone())
            .or_insert_with(BTreeMap::new)
            .insert(key.key, key.value);
    }
    
    // 4. 为每个文件筛选并写回使用的键
    for (lang, files) in keys_by_lang_and_file {
        if config.verbose {
            println!("正在处理语言: {}", lang);
        }
        for (file_path_str, defined_keys_map) in files {
            let file_path = PathBuf::from(file_path_str);
            if config.verbose {
                println!("  正在处理文件: {}", file_path.display());
            }

            // 只保留在源代码中使用过的键
            let filtered_keys: BTreeMap<String, String> = defined_keys_map
                .into_iter()
                .filter(|(key, _)| used_literal_keys.contains(key))
                .collect();

            if config.verbose {
                println!("    找到 {} 个使用中的键，准备写入文件。", filtered_keys.len());
            }
            
            // 5. 调用 parser 中的新函数写回文件
            parser::write_translation_file(&file_path, &filtered_keys)?;
        }
    }

    println!("提取成功！翻译文件已按使用的键进行更新。");
    Ok(())
}

/// 格式化翻译文件
fn run_format(config: &config::Config) -> Result<()> {
    // 1. 扫描源代码获取 used_keys
    let used_keys = scanner::scan_source_code(config)
        .context("扫描源代码以格式化失败")?;
    
    // 创建一个所有使用过的字面量键的有序集合
    let master_key_set: BTreeMap<String, ()> = used_keys
        .iter()
        .filter(|k| k.is_literal)
        .map(|k| (k.key.clone(), ()))
        .collect();

    if config.verbose {
        println!("找到 {} 个唯一的字面量使用键作为格式化基准。", master_key_set.len());
    }

    // 2. 解析翻译文件获取 defined_keys
    let defined_keys = parser::parse_translation_files(config)
        .context("解析翻译文件以格式化失败")?;

    // 3. 按语言和文件路径对已定义的键进行分组
    // HashMap<language, HashMap<file_path, HashMap<key, value>>>
    let mut keys_by_lang_and_file: HashMap<String, HashMap<String, HashMap<String, String>>> = HashMap::new();
    for key in defined_keys {
        keys_by_lang_and_file
            .entry(key.language.clone())
            .or_insert_with(HashMap::new)
            .entry(key.file_path.clone())
            .or_insert_with(HashMap::new)
            .insert(key.key, key.value);
    }
    
    // 4. 遍历每个文件，根据主键列表格式化并写回
    for (lang, files) in keys_by_lang_and_file {
        if config.verbose {
            println!("正在格式化语言: {}", lang);
        }
        for (file_path_str, defined_keys_map) in files {
            let file_path = PathBuf::from(file_path_str);
            if config.verbose {
                println!("  正在处理文件: {}", file_path.display());
            }

            let mut formatted_keys = BTreeMap::new();
            
            // 遍历主键列表，确保所有文件键序一致
            for (key, _) in &master_key_set {
                // 如果当前语言文件有这个键，则使用它的值，否则使用空字符串作为占位符
                let value = defined_keys_map.get(key).cloned().unwrap_or_default();
                formatted_keys.insert(key.clone(), value);
            }
            
            if config.verbose {
                println!("    文件将以 {} 个键进行格式化。", formatted_keys.len());
            }
            
            // 5. 调用 parser 中的函数写回文件
            parser::write_translation_file(&file_path, &formatted_keys)?;
        }
    }

    println!("格式化成功！翻译文件已排序和对齐。");
    Ok(())
}
