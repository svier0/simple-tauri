use std::path::Path;
use std::fs;

/// 下载文件
pub fn wget(url: &str) -> Result<(), String> {
	let parsed = Url::parse(url).unwrap();
    let fname = parsed
        .path_segments()
        .and_then(|s| s.last())
        .unwrap_or("index.html");
	wget_O(fname,url)
}

/// 下载文件 指定保存文件名
pub fn wget_O(fname: &str,url: &str) -> Result<(), String> {
    let p = Path::new(fname);
    let tmp_file = if p.is_absolute() {
        p.to_path_buf()
    } else {
        let tmp_dir = std::env::temp_dir();
        tmp_dir.join(fname)
    };

    let response = reqwest::blocking::get(&url)
        .map_err(|e| format!("下载失败: {}", e))?;
    if !response.status().is_success() {
        return Err(format!("HTTP {}: {}", response.status(), response.text().unwrap_or_default()));
    }

    let mut file = fs::File::create(&tmp_file)
        .map_err(|e| format!("创建临时文件失败: {}", e))?;
    let mut stream = response
        .bytes_stream()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e));
    std::io::copy(&mut stream, &mut file)
        .map_err(|e| format!("写入失败: {}", e))?;
    Ok(())
}