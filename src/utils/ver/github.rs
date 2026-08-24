
use super::https_agent;

/// 查询 github release 的最新正式版本号，失败返回空字符串（错误打到 stderr）
pub fn get_gh_latest_ver(gitrepo: &str) -> String {
    let url = format!("https://api.github.com/repos/{gitrepo}/releases/latest");
    let body = match https_agent().get(&url)
        .set("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/161.0.0.0 Safari/537.36")
        .call()
    {
        Ok(resp) => match resp.into_string() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("get_gh_latest_ver: 读取响应体失败: {e}");
                return String::new();
            }
        },
        Err(e) => {
            eprintln!("get_gh_latest_ver: 请求失败 url={url} err={e}");
            return String::new();
        }
    };
    match serde_json::from_str::<serde_json::Value>(&body) {
        Ok(v) => v["tag_name"]
            .as_str()
            .map(|s| s.trim_start_matches('v').to_string())
            .unwrap_or_default(),
        Err(e) => {
            eprintln!(
                "get_gh_latest_ver: JSON 解析失败: {e}; body(head)={}",
                &body.chars().take(200).collect::<String>()
            );
            String::new()
        }
    }
}

/// 查询 github release 的最新预览版本号，失败返回空字符串（错误打到 stderr）
pub fn get_gh_preview_ver(gitrepo: &str) -> String {
    let url = format!("https://api.github.com/repos/{gitrepo}/releases");
    let body = match https_agent().get(&url)
        .set("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/161.0.0.0 Safari/537.36")
        .call()
    {
        Ok(resp) => match resp.into_string() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("get_gh_preview_ver: 读取响应体失败: {e}");
                return String::new();
            }
        },
        Err(e) => {
            eprintln!("get_gh_preview_ver: 请求失败 url={url} err={e}");
            return String::new();
        }
    };
    match serde_json::from_str::<serde_json::Value>(&body) {
        Ok(v) => v[0]["tag_name"]
            .as_str()
            .map(|s| s.trim_start_matches('v').to_string())
            .unwrap_or_default(),
        Err(e) => {
            eprintln!(
                "get_gh_preview_ver: JSON 解析失败: {e}; body(head)={}",
                &body.chars().take(200).collect::<String>()
            );
            String::new()
        }
    }
}
