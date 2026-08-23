
use std::sync::Arc;

/// 所有 https 请求共用一个 agent：只加密，不校验对端证书（接受任何证书）
pub(crate) fn https_agent() -> ureq::Agent {
    let connector = native_tls::TlsConnector::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .expect("构建 TLS connector 失败");
    ureq::AgentBuilder::new()
        .tls_connector(Arc::new(connector))
        .build()
}

/// 查询最新版本号
pub fn get_latest_ver(pkgtype: &str,pkgname: &str) -> String {
    match pkgtype {
        "npm" => get_npm_latest_ver(pkgname),
        "github" => get_gh_latest_ver(pkgname),
        "github-preview" => get_gh_preview_ver(pkgname),
        _ => panic!("未知类型: {}", pkgtype),
    }
}

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
