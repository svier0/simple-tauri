
use simple_tauri_macros::sh2rs;

/// 解压远程zip到指定目录 (可选提取目录，默认提取全部)
pub fn unzip_remote(zipurl: &str, dir: &str, extract_dir: &str) -> Result<(), String> {
    // ---------- 下载 ----------
    let fname = url::Url::parse(zipurl)
        .map_err(|e| format!("解析URL失败: {}", e))?
        .path_segments()
        .and_then(|s| s.last())
        .unwrap_or("tmp.zip")
        .to_string();
    let tmp_file = std::env::temp_dir().join(&fname);
    sh2rs!("wget -O {} {}", tmp_file.display(), zipurl)?;

    // ---------- 清理目录 ----------
    sh2rs!("rm -rf {}", dir)
        .map_err(|e| format!("删除目录失败: {}", e))?;
    sh2rs!("mkdir -p {}", dir)
        .map_err(|e| format!("创建目录失败: {}", e))?;

    // ---------- 解压 ----------
    sh2rs!("unzip {} {} {}", tmp_file.display(), dir, extract_dir)
        .map_err(|e| format!("解压失败: {}", e))?;

    // ---------- 删除临时文件 ----------
    sh2rs!("rm {}", tmp_file.display())?;

    Ok(())
}

/// 相对路径转绝对路径
/// @param path &str 相对路径
/// @param root PathBuf 相对路径的起始目录
/// @return Pathbuf 返回绝对路径
pub fn path_rel2abs(path: &str,root: std::path::PathBuf) -> std::path::PathBuf {
    let p = std::path::Path::new(&path);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        root.join(&path)
    }
}