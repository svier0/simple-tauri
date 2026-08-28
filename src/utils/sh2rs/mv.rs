use std::fs;
use std::path::Path;

/// 移动文件或目录（跨设备时回退为 复制+删除）
pub fn mv(origin_path: &str, target_path: &str) -> Result<(), String> {
    let origin = Path::new(origin_path);
    if !origin.exists() {
        return Err(format!("源路径不存在: {}", origin_path));
    }

    let target = Path::new(target_path);
    let dest = if target.is_dir() {
        target.join(
            origin
                .file_name()
                .ok_or_else(|| format!("无法解析源路径名: {}", origin_path))?,
        )
    } else {
        target.to_path_buf()
    };

    match fs::rename(origin, &dest) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::CrossesDevices => {
            super::cp::cp(origin_path, target_path)?;
            super::rm::rm_r(origin_path)
        }
        Err(e) => Err(e.to_string()),
    }
}