
use super::https_agent;

/// 查询 npm 包的最新版本号，失败返回空字符串（错误打到 stderr）
pub fn get_npm_latest_ver(pkgname: &str) -> String {
    let url = format!("https://registry.npmjs.org/{}", pkgname.replace('/', "%2F"));
    let resp = match https_agent().get(&url)
        .set("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/161.0.0.0 Safari/537.36")
        .call()
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("get_npm_latest_ver: 请求失败 url={url} err={e}");
            return String::new();
        }
    };
    let status = resp.status();
    let body = match resp.into_string() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("get_npm_latest_ver: 读取响应体失败 status={status} err={e}");
            return String::new();
        }
    };
    match serde_json::from_str::<serde_json::Value>(&body) {
        Ok(v) => match v["dist-tags"]["latest"].as_str() {
            Some(s) => s.to_string(),
            None => {
                eprintln!(
                    "get_npm_latest_ver: status={status} 但响应无 dist-tags.latest; body(head)={}",
                    &body.chars().take(300).collect::<String>()
                );
                String::new()
            }
        },
        Err(e) => {
            eprintln!(
                "get_npm_latest_ver: status={status} JSON 解析失败: {e}; body(head)={}",
                &body.chars().take(300).collect::<String>()
            );
            String::new()
        }
    }
}