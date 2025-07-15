use serde::{Serialize, Deserialize};
use std::path::PathBuf;

fn default_true() -> bool {
    true
}

/// 应用程序配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 项目根目录
    pub project_path: PathBuf,
    /// 源代码目录
    pub src_dir: String,
    /// 翻译文件目录
    pub locales_dir: String,
    /// 警告阈值百分比
    pub threshold: f32,
    /// 忽略模式（正则表达式）
    pub ignore_pattern: Option<String>,
    /// 默认忽略翻译文件夹下 start_with TODO 的文件
    #[serde(default = "default_true")]
    pub ignore_todo_files: bool,
    /// 详细输出模式
    pub verbose: bool,
}

impl Config {
    /// 获取源代码目录的完整路径
    pub fn src_path(&self) -> PathBuf {
        self.project_path.join(&self.src_dir)
    }
    
    /// 获取翻译文件目录的完整路径
    pub fn locales_path(&self) -> PathBuf {
        self.project_path.join(&self.locales_dir)
    }
} 