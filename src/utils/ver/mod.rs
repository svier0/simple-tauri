
use std::sync::Arc;

mod npm;
mod github;
pub npm::{get_npm_latest_ver};
pub github::{get_gh_latest_ver,get_gh_preview_ver};

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
