mod scanner;
mod parser;
mod analyzer;
mod config;
mod report;

use clap::{Parser, Subcommand};
use anyhow::{Result, Context};
use std::path::PathBuf;
use walkdir::WalkDir;
use regex::Regex;
use std::fs;

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
}

fn main() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();
    
    // 创建配置
    let config = config::Config {
        project_path: cli.path,
        src_dir: cli.src_dir,
        locales_dir: cli.locales_dir,
        threshold: cli.threshold,
        ignore_pattern: cli.ignore_pattern,
        verbose: cli.verbose,
    };

    match cli.command.unwrap_or(Commands::Run { 
        format: "text".to_string(), 
        output: None 
    }) {
        Commands::Run { format, output } => {
            // 1. 扫描源代码，提取所有 t!() 宏调用中使用的键
            let used_keys = scanner::scan_source_code(&config)
                .context("扫描源代码失败")?;
            
            if config.verbose {
                println!("找到 {} 个使用中的翻译键", used_keys.len());
                println!("源代码目录: {}", config.src_path().display());
                
                // 输出扫描的文件列表
                println!("扫描的文件列表:");
                for entry in WalkDir::new(&config.src_path())
                    .follow_links(true)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().is_file())
                {
                    let path = entry.path();
                    if let Some(ext) = path.extension() {
                        if ext == "rs" {
                            println!("  - {}", path.display());
                        }
                    }
                }
                
                if !used_keys.is_empty() {
                    println!("使用中的翻译键详情:");
                    for key in &used_keys {
                        println!("  - {} ({}:{})", key.key, key.file_path, key.line_number);
                    }
                }
            }
            
            // 2. 解析翻译文件，提取所有定义的翻译键
            // 检查 rust_i18n::i18n!() 的调用并提取路径参数
            let mut locales_dir = config.locales_dir.clone();
            
            // 先尝试扫描代码中的 i18n!() 调用获取实际路径
            for entry in WalkDir::new(&config.src_path())
                .follow_links(true)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_file())
            {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext == "rs" {
                        if let Ok(content) = fs::read_to_string(path) {
                            // 查找 rust_i18n::i18n!("路径") 或 i18n!("路径")
                            let i18n_regex = Regex::new(r#"(?:rust_i18n::)?i18n!\s*\(\s*"([^"]+)"#).unwrap();
                            
                            if let Some(caps) = i18n_regex.captures(&content) {
                                if let Some(path_match) = caps.get(1) {
                                    let i18n_path = path_match.as_str();
                                    if config.verbose {
                                        println!("检测到 i18n! 路径参数: {}", i18n_path);
                                    }
                                    
                                    // 处理相对路径
                                    if i18n_path.starts_with("../") {
                                        let src_dir_path = config.project_path.join(&config.src_dir);
                                        let src_parent = src_dir_path.parent()
                                            .unwrap_or(&config.project_path);
                                        
                                        let abs_path = pathdiff::diff_paths(
                                            src_parent.join(i18n_path.trim_start_matches("../")),
                                            &config.project_path
                                        ).unwrap_or_else(|| PathBuf::from(i18n_path));
                                        
                                        locales_dir = abs_path.to_string_lossy().to_string();
                                        
                                        if config.verbose {
                                            println!("调整后的翻译文件路径: {}", locales_dir);
                                        }
                                        break;
                                    } else {
                                        locales_dir = i18n_path.to_string();
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // 使用调整后的翻译文件路径
            let mut adjusted_config = config.clone();
            adjusted_config.locales_dir = locales_dir;
            
            let defined_keys = parser::parse_translation_files(&adjusted_config)
                .context("解析翻译文件失败")?;
            
            if config.verbose {
                println!("找到 {} 个已定义的翻译键", defined_keys.len());
            }
            
            // 3. 比对两者，生成未使用翻译的报告
            let analysis_result = analyzer::analyze(&used_keys, &defined_keys, &config)
                .context("分析翻译键使用情况失败")?;
            
            // 4. 生成报告
            report::generate_report(&analysis_result, &format, output.as_deref(), &config)
                .context("生成报告失败")?;
            
            // 5. 如果未使用的翻译键百分比超过阈值，返回错误以便在 CI 中失败
            if analysis_result.unused_percentage > config.threshold {
                anyhow::bail!(
                    "未使用的翻译键比例 ({:.2}%) 超过阈值 ({:.2}%)",
                    analysis_result.unused_percentage,
                    config.threshold
                );
            }
        }
    }

    Ok(())
}
