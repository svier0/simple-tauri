use std::fs;

/// 创建目录（父目录必须已存在）
pub fn mkdir(dir: &str) -> Result<(), String> {
    fs::create_dir(dir).map_err(|e| e.to_string())
}

/// 创建目录（递归，自动创建中间层级）
pub fn mkdir_p(dir: &str) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|e| e.to_string())
}
