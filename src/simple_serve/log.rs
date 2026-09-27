use std::path::{Path, PathBuf};
use std::sync::{Mutex};

/// 日志文件路径
static PATH: Mutex<Option<PathBuf>> = Mutex::new(None);

pub fn set_log_path(p: impl AsRef<Path>) {
    let p = p.as_ref();
    let full_path = crate::utils::path_rel2abs(p, crate::simple_tray::resource_dir(""));
    *PATH.lock().unwrap() = Some(full_path.clone());
    if let Some(parent) = full_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("创建日志目录失败: {e}"))
                .ok();
        }
    }
}

pub fn get_log_path() -> PathBuf {
    // 先取出 clone 并释放锁，避免在持锁状态下调用 set_log_path 造成同线程死锁
    if let Some(p) = PATH.lock().unwrap().clone() {
        return p;
    }
    let default = PathBuf::from("log/server.log");
    set_log_path(&default);
    // set_log_path 存入的是绝对路径，重新读取返回
    PATH.lock().unwrap().clone().unwrap_or(default)
}

pub fn get_log() -> String {
    let path = get_log_path();
    std::fs::read_to_string(&path).unwrap_or_default()
}