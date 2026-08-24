
use super::local_ver::{check_local_ver,set_local_ver};
use super::work_dir::{get_work_dir};

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

/// 包类型
static PKG_TYPE: OnceLock<String> = OnceLock::new();
/// 包名称
static PKG_NAME: OnceLock<String> = OnceLock::new();
/// 自动更新
static AUTO_UPDATE: AtomicBool = AtomicBool::new(false);
/// 下载更新回调
static ENSURE_SERVER: OnceLock<fn(&str,&str) -> Result<(), String>> = OnceLock::new();
/// 解析VER为下载地址的闭包
static UPDATE_URL: OnceLock<fn(&str) -> String> = OnceLock::new();
/// 下载包提取目录
static EXTRACT_DIR: OnceLock<String> = OnceLock::new();

/// 设置包类型和名称
/// 包类型例如 npm github github_preview等
pub fn set_pkg(pkg_type: &str,pkg_name: &str){
    let _ = PKG_TYPE.set(pkg_type.to_string());
    let _ = PKG_NAME.set(pkg_name.to_string());
}

/// 开启自动更新
pub fn enable_auto_update(){
    AUTO_UPDATE.store(true, Ordering::SeqCst);
}

/// 设置解析ver为下载url的回调
pub fn set_download_url(update_url: fn(&str) -> String,extract_dir: &str) {
    UPDATE_URL.set(update_url).unwrap();
    let _ = EXTRACT_DIR.set(extract_dir.to_string());
}

/// 设置下载更新回调
/// 不调用本函数则需要调用set_download_url以使用默认的ensure_server_default
pub fn set_ensure_server(ensure_server: fn(&str,&str) -> Result<(), String>){
    ENSURE_SERVER.set(ensure_server).unwrap();
}

/// 更新指定版本的服务
/// 未调用 set_ensure_server 时的默认实现（按 set_download_url 解析出的地址下载并解压）
fn ensure_server_default(ver: &str,dir: &str) -> Result<(), String> {
    let url = UPDATE_URL.get().unwrap()(ver);
    let extract_dir = EXTRACT_DIR.get().unwrap();
    crate::utils::unzip_remote(&url,dir,extract_dir)?;
    Ok(())
}

/// 检查更新
/// 如果本地没有版本或者自动更新开启，则触发更新
/// 未调用 set_ensure_server 时需先调用 set_download_url 以使用默认更新逻辑
pub fn check_update(force: bool) -> Result<(), String> {
    let pkg_type = PKG_TYPE.get().expect("包类型未设置");
    let pkg_name = PKG_NAME.get().expect("包名称未设置");
    let auto_update = AUTO_UPDATE.load(Ordering::SeqCst) || force;
    let _ensure_server = match ENSURE_SERVER.get() {
        Some(f) => *f,
        None => ensure_server_default,
    };

    let mut local_ver = check_local_ver();
    if local_ver=="" || auto_update {
        let latest_ver = crate::utils::get_latest_ver(pkg_type,pkg_name);
        if latest_ver=="" {
            return Err(format!("检查版本号失败"));
        }
        if local_ver!=latest_ver {
            let _ = _ensure_server(&latest_ver,
                &get_work_dir(Some(latest_ver.as_str()))
            ).map_err(|e| format!("服务器(v{latest_ver})安装失败: {e}"))?;
            local_ver = latest_ver;
        }
    }
    set_local_ver(&local_ver);
    Ok(())
}