use std::fs;

/// 复制文件或目录（目录递归复制自身）
pub fn cp(origin_path: &str, target_path: &str) -> Result<(), String> {
    let origin = super::to_path(origin_path);
    if !origin.exists() {
        return Err(format!("源路径不存在: {}", origin_path));
    }

    let target = super::to_path(target_path);
    let dest = if target.is_dir() {
        target.join(
            origin
                .file_name()
                .ok_or_else(|| format!("无法解析源路径名: {}", origin_path))?,
        )
    } else {
        target.to_path_buf()
    };

    if origin.is_dir() {
        fs::create_dir_all(&dest).map_err(|e| e.to_string())?;
        for entry in fs::read_dir(origin).map_err(|e| e.to_string())? {
            let entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path();
            let child_dst = dest.join(entry.file_name());
            cp(
                &path.to_string_lossy(),
                &child_dst.to_string_lossy(),
            )?;
        }
        Ok(())
    } else {
        fs::copy(origin, &dest).map_err(|e| e.to_string()).map(|_| ())
    }
}