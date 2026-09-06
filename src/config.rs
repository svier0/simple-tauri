use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

pub use simple_tauri_macros::{get, get_or};

pub trait ConfigDefault<'a>: Sized {
    fn from_value(v: &'a serde_json::Value) -> Option<Self>;
}

impl<'a> ConfigDefault<'a> for i64 {
    fn from_value(v: &'a serde_json::Value) -> Option<Self> { v.as_i64() }
}

impl<'a> ConfigDefault<'a> for i32 {
    fn from_value(v: &'a serde_json::Value) -> Option<Self> { v.as_i64().map(|n| n as i32) }
}

impl<'a> ConfigDefault<'a> for i16 {
    fn from_value(v: &'a serde_json::Value) -> Option<Self> { v.as_i64().map(|n| n as i16) }
}

impl<'a> ConfigDefault<'a> for i8 {
    fn from_value(v: &'a serde_json::Value) -> Option<Self> { v.as_i64().map(|n| n as i8) }
}

impl<'a> ConfigDefault<'a> for u64 {
    fn from_value(v: &'a serde_json::Value) -> Option<Self> { v.as_u64() }
}

impl<'a> ConfigDefault<'a> for u32 {
    fn from_value(v: &'a serde_json::Value) -> Option<Self> { v.as_u64().map(|n| n as u32) }
}

impl<'a> ConfigDefault<'a> for u16 {
    fn from_value(v: &'a serde_json::Value) -> Option<Self> { v.as_u64().map(|n| n as u16) }
}

impl<'a> ConfigDefault<'a> for u8 {
    fn from_value(v: &'a serde_json::Value) -> Option<Self> { v.as_u64().map(|n| n as u8) }
}

impl<'a> ConfigDefault<'a> for f64 {
    fn from_value(v: &'a serde_json::Value) -> Option<Self> { v.as_f64() }
}

impl<'a> ConfigDefault<'a> for bool {
    fn from_value(v: &'a serde_json::Value) -> Option<Self> { v.as_bool() }
}

impl<'a> ConfigDefault<'a> for String {
    fn from_value(v: &'a serde_json::Value) -> Option<Self> { v.as_str().map(|s| s.to_string()) }
}

impl<'a> ConfigDefault<'a> for &'a str {
    fn from_value(v: &'a serde_json::Value) -> Option<Self> { v.as_str() }
}

/// 库内置兜底默认配置（JSONC）
const FALLBACK: &str = r#"
{
    // 开机自启
    "auto_start": false,
    // 自动运行
    "auto_run": false,
    // 静默启动
    "silent_launch": false,
    // 自动更新
    "auto_update": false,
}
"#;

/// 默认配置原始文本
static DEFAULT_RAW: OnceLock<String> = OnceLock::new();
/// 用户配置文件原始文本（保留注释和格式）
static RAW: OnceLock<Mutex<String>> = OnceLock::new();
/// 用户配置文件路径（用于回写）
static PATH: OnceLock<PathBuf> = OnceLock::new();
/// 合并后的最终配置
static CONFIG: OnceLock<Mutex<serde_json::Value>> = OnceLock::new();

fn config() -> &'static Mutex<serde_json::Value> {
    CONFIG.get_or_init(|| {
        let val: serde_json::Value = json5::from_str(FALLBACK)
            .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
        Mutex::new(val)
    })
}

fn raw() -> Option<&'static Mutex<String>> {
    RAW.get()
}

/// 设置默认配置（JSONC 格式字符串），会覆盖当前配置
pub fn set_default(jsonc: &str) {
    let val: serde_json::Value = json5::from_str(jsonc)
        .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
    *config().lock().unwrap() = val;
    let _ = DEFAULT_RAW.set(jsonc.to_string());
}

/// 获取默认配置原始字符串（调用方设置的，未设置则返回兜底配置）
pub fn get_default_raw() -> &'static str {
    DEFAULT_RAW.get().map(|s| s.as_str()).unwrap_or(FALLBACK)
}

/// 加载用户配置文件（JSONC/JSON5 格式），合并到当前配置（用户覆盖默认）
pub fn load(path: impl AsRef<Path>) -> Result<(), String> {
    let p = path.as_ref();
    let full_path = if p.is_absolute() {
        p.to_path_buf()
    } else {
        crate::simple_tray::resource_dir("").join(p)
    };
    let content = std::fs::read_to_string(&full_path)
        .map_err(|e| format!("读取配置文件失败: {e}"))?;
    let user: serde_json::Value = json5::from_str(&content)
        .map_err(|e| format!("解析配置文件失败: {e}"))?;
    let mut cfg = config().lock().unwrap();
    *cfg = merge(&cfg, &user);
    let _ = PATH.set(full_path);
    let _ = RAW.set(Mutex::new(content));
    Ok(())
}

/// 获取指定键的配置值，无则返回默认值
pub fn get(key: &str, default: impl Into<serde_json::Value>) -> serde_json::Value {
    config().lock().unwrap().get(key).cloned().unwrap_or(default.into())
}

/// 获取所有配置键名
pub fn keys() -> Vec<String> {
    let cfg = config().lock().unwrap();
    match *cfg {
        serde_json::Value::Object(ref map) => map.keys().cloned().collect(),
        _ => vec![],
    }
}

/// 获取全部配置
pub fn all() -> serde_json::Value {
    config().lock().unwrap().clone()
}

/// 设置指定键的配置值并回写到文件（保留原文件格式）
pub fn set(key: &str, value: impl Into<serde_json::Value>) -> Result<(), String> {
    let value = value.into();
    {
        let mut cfg = config().lock().unwrap();
        if let serde_json::Value::Object(ref mut map) = *cfg {
            map.insert(key.to_string(), value.clone());
        }
    }
    let r = raw().ok_or("未加载配置文件，无法回写")?;
    let mut text = r.lock().unwrap();
    replace_value(&mut text, key, &value);
    let path = PATH.get().ok_or("未加载配置文件，无法回写")?;
    std::fs::write(path, text.as_str())
        .map_err(|e| format!("写入配置文件失败: {e}"))
}

/// 深度合并：b 覆盖 a
fn merge(a: &serde_json::Value, b: &serde_json::Value) -> serde_json::Value {
    match (a, b) {
        (serde_json::Value::Object(a), serde_json::Value::Object(b)) => {
            let mut merged = a.clone();
            for (k, v) in b {
                merged.insert(k.clone(), merge(a.get(k).unwrap_or(&serde_json::Value::Null), v));
            }
            serde_json::Value::Object(merged)
        }
        _ => b.clone(),
    }
}

/// 在原始文本中替换指定 key 的值（保留注释和格式）
fn replace_value(text: &mut String, key: &str, value: &serde_json::Value) {
    let new_val = value_to_string(value);
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(&format!("\"{key}\"")) {
            if let Some(colon_pos) = trimmed.find(':') {
                let after_colon = &trimmed[colon_pos + 1..].trim_start();
                let val_end = find_value_end(after_colon);
                if val_end > 0 {
                    let old_val = &after_colon[..val_end];
                    let byte_start = text.find(old_val);
                    if let Some(start) = byte_start {
                        let byte_end = start + old_val.len();
                        text.replace_range(start..byte_end, &new_val);
                        return;
                    }
                }
            }
        }
    }
}

/// 值转 JSON 字符串
fn value_to_string(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
        other => other.to_string(),
    }
}

/// 找到值的结束位置（跳过引号字符串）
fn find_value_end(s: &str) -> usize {
    let s = s.trim_start();
    if s.starts_with('"') {
        let mut escaped = false;
        for (i, c) in s.char_indices().skip(1) {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                return i + 1;
            }
        }
        0
    } else {
        // 非字符串：取到逗号、换行、} 或末尾
        for (i, c) in s.char_indices() {
            if c == ',' || c == '}' || c == '\n' {
                return i;
            }
        }
        s.len()
    }
}
