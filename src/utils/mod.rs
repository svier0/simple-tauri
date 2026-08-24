mod ver;
mod sh2rs;

pub use simple_tauri_macros::sh2rs;
pub use ver::*;
pub use sh2rs::*;

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
    sh2rs!("rm {}", tmp_file.display())
}