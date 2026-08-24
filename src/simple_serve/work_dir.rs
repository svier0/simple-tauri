
use super::local_ver::{get_local_ver};

use std::sync::{OnceLock};

/// 服务器工作目录规则（含 <ver> 占位符，OnceLock 只设一次）
static WORK_DIR: OnceLock<String> = OnceLock::new();

/// 设置服务器工作目录规则（含 <ver> 占位符，只设一次）
pub fn set_work_dir(rule: &str) {
    let _ = WORK_DIR.set(rule.to_string());
}

/// 获取工作目录的绝对路径（String）
/// work_dir.replace("<ver>", ver)，ver 为 None 时使用 LOCAL_VER
/// 读取全局 WORK_DIR（由 set_work_dir 设置）；未调用 set_work_dir 会直接 panic 报错
pub fn get_work_dir(ver: Option<&str>) -> String {
    let rule = WORK_DIR
        .get()
        .expect("set_work_dir 必须先于 get_work_dir 调用");
    let v = ver.map(|s| s.to_string())
        .or_else(|| Some(get_local_ver()))
        .unwrap_or_default();
    crate::simple_tray::resource_dir(&rule.replace("<ver>", &v))
        .to_string_lossy()
        .into_owned()
}