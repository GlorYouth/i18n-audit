use i18n_audit::config;
use clap::{Parser, Subcommand};
use anyhow::Result;
use std::path::PathBuf;
use i18n_audit::actions;

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
    actions::adjust_locales_path(&mut config)?;

    match cli.command.unwrap_or(Commands::Run { 
        format: "text".to_string(), 
        output: None 
    }) {
        Commands::Run { format, output } => {
            actions::run_audit_command(&config, &format, output.as_deref())?
        }
        Commands::Extract => {
            actions::run_extract_command(&config)?
        }
        Commands::Format => {
            actions::run_format_command(&config)?
        }
    }

    Ok(())
}
