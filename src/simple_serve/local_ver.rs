
use super::work_dir::{get_work_dir};

use std::sync::{Mutex};
use std::path::Path;

/// 本地版本号（set_local_ver 设置后替换 <ver>）
static LOCAL_VER: Mutex<Option<String>> = Mutex::new(None);

/// 设置本地版本号（用于替换 work_dir 中的 <ver>）
pub fn set_local_ver(ver: &str) {
    *LOCAL_VER.lock().unwrap() = Some(ver.to_string());
}

/// 获取设置的本地版本号
pub fn get_local_ver() -> String {
    LOCAL_VER.lock().unwrap().clone().unwrap_or_default()
}

/// 检测本地版本号：用 WORK_DIR 规则扫描目录，返回匹配的版本号；没有返回空字符串
pub fn check_local_ver() -> String {
    let rule = get_work_dir(None);
    // 1. 相对路径拼接resourcedir转绝对路径
    let abs = crate::utils::path_rel2abs(&rule,crate::simple_tray::resource_dir(""));
    // 2. 绝对路径取父目录
    let parent = abs.parent().unwrap_or(Path::new("/"));
    if !parent.exists() {
        return String::new();
    }
    // 3. ver 格式
    let p = Path::new(&rule);
    let fname = match p.file_name().and_then(|n| n.to_str()) {
        Some(f) => f,
        None => return String::new(),
    };
    let ver_pos = match fname.find("<ver>") {
        Some(pos) => pos,
        None => return String::new(),
    };
    let prefix = &fname[..ver_pos];
    let suffix = &fname[ver_pos + 5..];
    // 4. 遍历父目录下所有目录，寻找符合 rule 格式的
    let entries = match std::fs::read_dir(parent) {
        Ok(e) => e,
        Err(_) => return String::new(),
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !name.starts_with(prefix) {
            continue;
        }
        let rest = &name[prefix.len()..];
        if let Some(ver) = rest.strip_suffix(suffix) {
            if !ver.is_empty() && entry.path().is_dir() {
                return ver.to_string();
            }
        }
    }
    String::new()
}