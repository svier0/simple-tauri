use std::fs;
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
    let tmp_file = super::to_path(fname);

    let resp = crate::utils::ver::https_agent()
        .get(url)
        .call()
        .map_err(|e| format!("下载失败: {}", e))?;

    let mut file = fs::File::create(&tmp_file)
        .map_err(|e| format!("创建临时文件失败: {} {}",&tmp_file.display(), e))?;
    let mut reader = resp.into_reader();
    std::io::copy(&mut reader, &mut file).map_err(|e| format!("写入失败: {}", e))?;
    Ok(())
}

/// 读取远程文件 不落盘
#[allow(non_snake_case)]
pub fn wget_O_(url: &str) -> Result<(), String> {
    let resp = crate::utils::ver::https_agent()
        .get(url)
        .set("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/161.0.0.0 Safari/537.36")
        .call()
        .map_err(|e| format!("wget: 请求失败: url={url} err={e}"))?;
    let status = resp.status();
    let body = resp.into_string()
        .map_err(|e| format!("wget: 读取响应体失败: status={status} err={e}"))?;
    super::set_stdout_buffer(&body);
    Ok(())
}