
/// 查询最新版本号
pub fn get_latest_ver(pkgtype: &str,pkgname: &str) -> String {
    match pkgtype {
        "npm" => get_npm_latest_ver(pkgname),
        "github" => get_gh_latest_ver(pkgname),
        "github-preview" => get_gh_preview_ver(pkgname),
        _ => panic!("未知类型: {}", pkgtype),
    }
}

/// 查询 npm 包的最新版本号，失败返回空字符串
pub fn get_npm_latest_ver(pkgname: &str) -> String {
    let url = format!("https://registry.npmjs.org/{}", pkgname.replace('/', "%2F"));
    let body = match ureq::get(&url)
        .set("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/161.0.0.0 Safari/537.36")
        .call()
    {
        Ok(resp) => match resp.into_string() {
            Ok(s) => s,
            Err(_) => return String::new(),
        },
        Err(_) => return String::new(),
    };
    serde_json::from_str::<serde_json::Value>(&body)
        .ok()
        .and_then(|v| v["dist-tags"]["latest"].as_str().map(|s| s.to_string()))
        .unwrap_or_default()
}

/// 查询 github release 的最新正式版本号，失败返回空字符串
pub fn get_gh_latest_ver(gitrepo: &str) -> String {
    let url = format!("https://api.github.com/repos/{gitrepo}/releases/latest");
    let body = match ureq::get(&url)
        .set("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/161.0.0.0 Safari/537.36")
        .call()
    {
        Ok(resp) => match resp.into_string() {
            Ok(s) => s,
            Err(_) => return String::new(),
        },
        Err(_) => return String::new(),
    };
    let ver = serde_json::from_str::<serde_json::Value>(&body)
        .ok()
        .and_then(|v| v["tag_name"].as_str().map(|s| s.to_string()))
        .unwrap_or_default();
    ver.trim_start_matches('v').to_string()
}

/// 查询 github release 的最新预览版本号，失败返回空字符串
pub fn get_gh_preview_ver(gitrepo: &str) -> String {
    let url = format!("https://api.github.com/repos/{gitrepo}/releases");
    let body = match ureq::get(&url)
        .set("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/161.0.0.0 Safari/537.36")
        .call()
    {
        Ok(resp) => match resp.into_string() {
            Ok(s) => s,
            Err(_) => return String::new(),
        },
        Err(_) => return String::new(),
    };
    let ver = serde_json::from_str::<serde_json::Value>(&body)
        .ok()
        .and_then(|v| v[0]["tag_name"].as_str().map(|s| s.to_string()))
        .unwrap_or_default();
    ver.trim_start_matches('v').to_string()
}
