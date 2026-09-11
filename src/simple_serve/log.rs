use std::path::{Path, PathBuf};
use std::sync::{Mutex};

/// 日志文件路径
static PATH: Mutex<Option<PathBuf>> = Mutex::new(None);

pub fn set_log_path(p: impl AsRef<Path>) {
    let p = p.as_ref();
    let full_path = if p.is_absolute() {
        p.to_path_buf()
    } else {
        crate::simple_tray::resource_dir("").join(p)
    };
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
    PATH.lock()
        .unwrap()
        .clone()
        .unwrap_or_else(|| {
            let default = PathBuf::from("log/server.log");
            set_log_path(&default);
            default
        })
}

pub fn get_log() -> String {
    let path = get_log_path();
    std::fs::read_to_string(&path).unwrap_or_default()
}