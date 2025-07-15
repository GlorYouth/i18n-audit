use anyhow::{Result, Context, bail};
use log::{info, debug};
use serde::{Serialize, Deserialize};
use serde_yaml;
use serde_json;
use toml;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

use crate::config::Config;

/// 表示一个翻译键的定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefinedKey {
    /// 翻译键
    pub key: String,
    /// 语言代码
    pub language: String,
    /// 翻译值
    pub value: String,
    /// 所在文件路径
    pub file_path: String,
}

/// 解析翻译文件，提取所有定义的翻译键
pub fn parse_translation_files(config: &Config) -> Result<Vec<DefinedKey>> {
    let locales_path = config.locales_path();
    info!("正在解析翻译文件目录: {}", locales_path.display());
    
    let mut defined_keys = Vec::new();
    
    // 支持的翻译文件扩展名
    let supported_extensions = [".yml", ".yaml", ".json", ".toml"];
    
    for entry in WalkDir::new(&locales_path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        
        // 如果启用了忽略 TODO 文件，则跳过
        if config.ignore_todo_files {
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.to_lowercase().starts_with("todo") {
                    debug!("忽略 TODO 文件: {}", path.display());
                    continue;
                }
            }
        }
        
        if path.is_file() && path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| supported_extensions.iter().any(|&e| e == &format!(".{}", ext)))
            .unwrap_or(false) 
        {
            debug!("处理翻译文件: {}", path.display());
            
            // 从文件名或目录结构推断语言代码
            let language = extract_language_from_path(path, &locales_path)?;
            
            // 读取文件内容
            let relative_path = path.strip_prefix(&config.project_path)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();
                
            let content = fs::read_to_string(path)
                .with_context(|| format!("无法读取文件: {}", path.display()))?;
                
            // 根据文件扩展名选择合适的解析方法
            match path.extension().and_then(|ext| ext.to_str()) {
                Some("yml") | Some("yaml") => {
                    parse_yaml(&content, &language, &relative_path, &mut defined_keys)?;
                }
                Some("json") => {
                    parse_json(&content, &language, &relative_path, &mut defined_keys)?;
                }
                Some("toml") => {
                    parse_toml(&content, &language, &relative_path, &mut defined_keys)?;
                }
                _ => {
                    // 不应该发生，因为我们已经过滤了文件扩展名
                    bail!("不支持的文件类型: {}", path.display());
                }
            }
        }
    }
    
    info!("解析完成，找到 {} 个已定义的翻译键", defined_keys.len());
    
    Ok(defined_keys)
}

/// 从文件路径推断语言代码
fn extract_language_from_path(path: &Path, locales_path: &Path) -> Result<String> {
    // 首先尝试从文件名推断
    if let Some(file_name) = path.file_stem().and_then(|s| s.to_str()) {
        // 如果文件名就是语言代码 (例如：en.yml, zh-CN.json)
        if file_name.len() <= 5 && !file_name.contains('.') {
            return Ok(file_name.to_string());
        }
    }
    
    // 然后尝试从父目录推断
    if let Some(parent) = path.parent() {
        if let Ok(relative) = parent.strip_prefix(locales_path) {
            if let Some(first_dir) = relative.components().next() {
                if let Some(lang) = first_dir.as_os_str().to_str() {
                    if lang.len() <= 5 && !lang.contains('.') {
                        return Ok(lang.to_string());
                    }
                }
            }
        }
    }
    
    // 如果无法推断，使用默认值
    debug!("无法从路径推断语言代码: {}，使用默认值 'unknown'", path.display());
    Ok("unknown".to_string())
}

/// 从嵌套哈希表中提取所有键值对，键使用点分隔
fn extract_keys_from_map(
    map: &HashMap<String, serde_yaml::Value>,
    prefix: &str,
    language: &str,
    file_path: &str,
    defined_keys: &mut Vec<DefinedKey>
) {
    for (key, value) in map {
        let full_key = if prefix.is_empty() {
            key.clone()
        } else {
            format!("{}.{}", prefix, key)
        };
        
        match value {
            serde_yaml::Value::Mapping(nested_map) => {
                // 递归处理嵌套映射
                let nested_map_typed: HashMap<String, serde_yaml::Value> = nested_map.iter()
                    .filter_map(|(k, v)| {
                        if let serde_yaml::Value::String(key_str) = k {
                            Some((key_str.clone(), v.clone()))
                        } else {
                            None
                        }
                    })
                    .collect();
                
                extract_keys_from_map(&nested_map_typed, &full_key, language, file_path, defined_keys);
            }
            serde_yaml::Value::String(val) => {
                // 添加叶节点键值对
                defined_keys.push(DefinedKey {
                    key: full_key.clone(),
                    language: language.to_string(),
                    value: val.clone(),
                    file_path: file_path.to_string(),
                });
            }
            _ => {
                // 其他类型的值，转换为字符串
                defined_keys.push(DefinedKey {
                    key: full_key.clone(),
                    language: language.to_string(),
                    value: format!("{:?}", value),  // 使用 Debug trait 而不是 Display
                    file_path: file_path.to_string(),
                });
            }
        }
    }
}

/// 从 JSON 值中提取所有键值对
fn extract_keys_from_json(
    value: &serde_json::Value,
    prefix: &str,
    language: &str,
    file_path: &str,
    defined_keys: &mut Vec<DefinedKey>
) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, val) in map {
                let full_key = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", prefix, key)
                };
                
                match val {
                    serde_json::Value::Object(_) => {
                        // 递归处理嵌套对象
                        extract_keys_from_json(val, &full_key, language, file_path, defined_keys);
                    }
                    serde_json::Value::String(s) => {
                        // 显式处理字符串，获取原始值
                        defined_keys.push(DefinedKey {
                            key: full_key.clone(),
                            language: language.to_string(),
                            value: s.clone(),
                            file_path: file_path.to_string(),
                        });
                    }
                    _ => {
                        // 对其他类型（数字、布尔等）使用 to_string
                        defined_keys.push(DefinedKey {
                            key: full_key.clone(),
                            language: language.to_string(),
                            value: val.to_string(),
                            file_path: file_path.to_string(),
                        });
                    }
                }
            }
        }
        _ => {
            // 不应该发生，因为顶层应该是一个对象
            debug!("JSON 值不是对象: {}", value);
        }
    }
}

/// 从 TOML 值中提取所有键值对
fn extract_keys_from_toml(
    value: &toml::Value,
    prefix: &str,
    language: &str,
    file_path: &str,
    defined_keys: &mut Vec<DefinedKey>
) {
    match value {
        toml::Value::Table(map) => {
            for (key, val) in map {
                let full_key = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", prefix, key)
                };
                
                match val {
                    toml::Value::Table(_) => {
                        // 递归处理嵌套表
                        extract_keys_from_toml(val, &full_key, language, file_path, defined_keys);
                    }
                    toml::Value::String(s) => {
                        // 显式处理字符串，获取原始值
                        defined_keys.push(DefinedKey {
                            key: full_key.clone(),
                            language: language.to_string(),
                            value: s.clone(),
                            file_path: file_path.to_string(),
                        });
                    }
                    _ => {
                        // 对其他类型（数字、布尔等）使用 to_string
                        defined_keys.push(DefinedKey {
                            key: full_key.clone(),
                            language: language.to_string(),
                            value: val.to_string(),
                            file_path: file_path.to_string(),
                        });
                    }
                }
            }
        }
        _ => {
            // 不应该发生，因为顶层应该是一个表
            debug!("TOML 值不是表: {}", value);
        }
    }
}

/// 解析 YAML 文件
fn parse_yaml(content: &str, language: &str, file_path: &str, defined_keys: &mut Vec<DefinedKey>) -> Result<()> {
    let root: serde_yaml::Value = serde_yaml::from_str(content)
        .with_context(|| format!("无法解析 YAML 文件: {}", file_path))?;
        
    if let serde_yaml::Value::Mapping(map) = root {
        // 转换键为字符串类型
        let mut map_typed: HashMap<String, serde_yaml::Value> = HashMap::new();
        for (k, v) in map.iter() {
            if let Some(key_str) = k.as_str() {
                map_typed.insert(key_str.to_string(), v.clone());
            }
        }

        // 如果顶层键是语言本身，则从下一层开始
        if map_typed.len() == 1 && map_typed.contains_key(language) {
            if let Some(serde_yaml::Value::Mapping(nested_map)) = map_typed.get(language) {
                let nested_map_typed: HashMap<String, serde_yaml::Value> = nested_map.iter()
                    .filter_map(|(k, v)| k.as_str().map(|s| (s.to_string(), v.clone())))
                    .collect();
                extract_keys_from_map(&nested_map_typed, "", language, file_path, defined_keys);
            }
        } else {
            extract_keys_from_map(&map_typed, "", language, file_path, defined_keys);
        }
    } else {
        bail!("YAML 顶层必须是映射/对象: {}", file_path);
    }
    
    Ok(())
}

/// 解析 JSON 文件
fn parse_json(content: &str, language: &str, file_path: &str, defined_keys: &mut Vec<DefinedKey>) -> Result<()> {
    let root: serde_json::Value = serde_json::from_str(content)
        .with_context(|| format!("无法解析 JSON 文件: {}", file_path))?;

    if let serde_json::Value::Object(map) = root {
        // 如果顶层键是语言本身，则从下一层开始
        if map.len() == 1 && map.contains_key(language) {
            if let Some(serde_json::Value::Object(nested_map)) = map.get(language) {
                let value = serde_json::Value::Object(nested_map.clone());
                extract_keys_from_json(&value, "", language, file_path, defined_keys);
            }
        } else {
            let value = serde_json::Value::Object(map);
            extract_keys_from_json(&value, "", language, file_path, defined_keys);
        }
    } else {
        bail!("JSON 顶层必须是对象: {}", file_path);
    }
    
    Ok(())
}

/// 解析 TOML 文件
fn parse_toml(content: &str, language: &str, file_path: &str, defined_keys: &mut Vec<DefinedKey>) -> Result<()> {
    let root: toml::Value = toml::from_str(content)
        .with_context(|| format!("无法解析 TOML 文件: {}", file_path))?;

    if let toml::Value::Table(table) = root {
        // 如果顶层键是语言本身，则从下一层开始
        if table.len() == 1 && table.contains_key(language) {
            if let Some(toml::Value::Table(nested_table)) = table.get(language) {
                let value = toml::Value::Table(nested_table.clone());
                extract_keys_from_toml(&value, "", language, file_path, defined_keys);
            }
        } else {
            let value = toml::Value::Table(table);
            extract_keys_from_toml(&value, "", language, file_path, defined_keys);
        }
    } else {
        bail!("TOML 顶层必须是表: {}", file_path);
    }

    Ok(())
} 

/// 将扁平化的键值对（例如 "a.b.c": "value"）转换为嵌套的 serde_json::Value
fn unflatten_to_json_value(
    flat_keys: &BTreeMap<String, String>
) -> serde_json::Value {
    let mut root = serde_json::Value::Object(serde_json::Map::new());

    for (full_key, value) in flat_keys {
        let mut parts: Vec<&str> = full_key.split('.').collect();
        let leaf_key = parts.pop().unwrap();
        
        let mut current_level = &mut root;

        for part in parts {
            let next_level = if let serde_json::Value::Object(map) = current_level {
                map.entry(part.to_string())
                    .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()))
            } else {
                continue;
            };
            current_level = next_level;
        }

        if let serde_json::Value::Object(map) = current_level {
            map.insert(
                leaf_key.to_string(),
                serde_json::Value::String(value.clone()),
            );
        }
    }

    root
}

/// 将翻译键写入文件，覆盖现有内容
pub fn write_translation_file(
    file_path: &Path,
    keys: &BTreeMap<String, String>
) -> Result<()> {
    let extension = file_path.extension().and_then(|s| s.to_str()).unwrap_or("");

    let nested_json = unflatten_to_json_value(keys);

    let content = match extension {
        "yml" | "yaml" => {
            // JSON 是 YAML 的子集，所以我们可以将 JSON Value 转换为 YAML Value
            let yaml_value: serde_yaml::Value = serde_yaml::from_str(&serde_json::to_string(&nested_json)?)?;
            serde_yaml::to_string(&yaml_value)?
        }
        "json" => {
            serde_json::to_string_pretty(&nested_json)?
        }
        "toml" => {
            // 使用 `try_from` 将 `serde_json::Value` 转换为 `toml::Value`
            let toml_value = toml::Value::try_from(nested_json)
                .with_context(|| "无法将中间值转换为 TOML 格式")?;
            toml::to_string_pretty(&toml_value)?
        }
        _ => bail!("不支持的文件扩展名: {}", extension),
    };

    fs::write(file_path, content)
        .with_context(|| format!("无法写入文件: {}", file_path.display()))?;

    info!("成功覆盖文件: {}", file_path.display());
    Ok(())
} 