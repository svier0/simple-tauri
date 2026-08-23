use std::fs;
use std::path::Path;
use url::Url;

/// 下载文件
pub fn wget(url: &str) -> Result<(), String> {
    let parsed = Url::parse(url).map_err(|e| format!("解析URL失败: {}", e))?;
    let fname = parsed
        .path_segments()
        .and_then(|s| s.last())
        .filter(|s| !s.is_empty())
        .unwrap_or("index.html")
        .to_string();
    wget_O(&fname, url)
}

/// 下载文件 指定保存文件名
#[allow(non_snake_case)]
pub fn wget_O(fname: &str, url: &str) -> Result<(), String> {
    let p = Path::new(fname);
    let tmp_file = if p.is_absolute() {
        p.to_path_buf()
    } else {
        let tmp_dir = std::env::temp_dir();
        tmp_dir.join(fname)
    };

    let resp = crate::utils::ver::https_agent()
        .get(url)
        .call()
        .map_err(|e| format!("下载失败: {}", e))?;

    let mut file = fs::File::create(&tmp_file)
        .map_err(|e| format!("创建临时文件失败: {}", e))?;
    let mut reader = resp.into_reader();
    std::io::copy(&mut reader, &mut file).map_err(|e| format!("写入失败: {}", e))?;
    Ok(())
}
