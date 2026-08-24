use std::fs;
use std::path::Path;
use url::Url;
use std::sync::{Mutex};

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

static WGET_BODY_RESULT: Mutex<Option<String>> = Mutex::new(None);

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
    *WGET_BODY_RESULT.lock().unwrap() = Some(body);
    Ok(())
}

pub fn get_wget_result(_type:&str) -> String {
    let mut r = String::new();
    if _type=="body" {
        r = WGET_BODY_RESULT
            .lock()
            .unwrap()
            .clone()
            .unwrap_or_default();
        *WGET_BODY_RESULT.lock().unwrap() = None;
    }
    r
}