use std::path::Path;
use std::fs;

/// 删除文件/目录
pub fn rm(target: &str) -> Result<(), String> {
    let p = Path::new(target);
    if !p.exists() {
        return Ok(())
    }
    if p.is_dir() {
        fs::remove_dir(target).map_err(|e| e.to_string())
    } else if p.is_file() {
        fs::remove_file(target).map_err(|e| e.to_string())
    } else {
        Err(format!("无法识别的文件类型: {}", target))
    }
}

/// 删除文件/目录（递归）
pub fn rm_r(target: &str) -> Result<(), String> {
    let p = Path::new(target);
    if !p.exists() {
        return Ok(())
    }
    if p.is_dir() {
    	fs::remove_dir_all(target).map_err(|e| e.to_string())
    } else if p.is_file() {
        fs::remove_file(target).map_err(|e| e.to_string())
    } else {
        Err(format!("无法识别的文件类型: {}", target))
    }
}
