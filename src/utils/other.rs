
use simple_tauri_macros::sh2rs;

/// 解压远程zip到指定目录 (可选提取目录，默认提取全部)
pub fn unzip_remote(zipurl: &str, dir: &str, extract_dir: &str) -> Result<(), String> {
    // ---------- 下载 ----------
    let fname = url::Url::parse(zipurl)
        .map_err(|e| format!("解析URL失败: {}", e))?
        .path_segments()
        .and_then(|s| s.last())
        .unwrap_or("tmp.zip")
        .to_string();
    let tmp_file = std::env::temp_dir().join(&fname);
    sh2rs!("wget -O {} {}", tmp_file.display(), zipurl)?;

    // ---------- 清理目录 ----------
    sh2rs!("rm -rf {}", dir)
        .map_err(|e| format!("删除目录失败: {}", e))?;
    sh2rs!("mkdir -p {}", dir)
        .map_err(|e| format!("创建目录失败: {}", e))?;

    // ---------- 解压 ----------
    sh2rs!("unzip {} {} {}", tmp_file.display(), dir, extract_dir)
        .map_err(|e| format!("解压失败: {}", e))?;

    // ---------- 删除临时文件 ----------
    sh2rs!("rm {}", tmp_file.display())?;

    Ok(())
}

/// 相对路径转绝对路径
/// @param path &str 相对路径
/// @param root PathBuf 相对路径的起始目录
/// @return Pathbuf 返回绝对路径
pub fn path_rel2abs(path: &str,root: std::path::PathBuf) -> std::path::PathBuf {
    let p = std::path::Path::new(&path);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        root.join(&path)
    }
}

/// 异步线程回调转为同步阻塞
/// 例: sync_call(|ok,err|{ ok(); },1)?;
/// 提示: 当心同线程死锁
pub fn sync_call<F, O, E>(f: F,timeout_secs: u64) -> Result<(), String>
where
    F: FnOnce(O, E) -> Result<(), String>,
    O: FnOnce(),
    E: FnOnce(String),
{
    let timeout = std::time::Duration::from_secs(timeout_secs);

    let (tx, rx) = std::sync::mpsc::channel::<Result<(), String>>();
    let tx_err = tx.clone();

    let ok = || {
        let _ = tx.send(Ok(()));
    };
    let err = |msg: String| {
        let _ = tx_err.send(Err(msg));
    };

    let _ = f(ok,err)?;

    rx.recv_timeout(timeout)
            .map_err(|e| match e {
                std::sync::mpsc::RecvTimeoutError::Timeout => format!("callback timed out after {:?}", timeout),
                std::sync::mpsc::RecvTimeoutError::Disconnected => "callback dropped before completing".into(),
            })
            .and_then(|r| r)?
}

/// 异步事件回调转为同步阻塞
pub async fn async_call<F, O, E>(f: F, timeout_secs: u64) -> Result<(), String>
where
    F: FnOnce(O, E) -> Result<(), String>,
    O: FnOnce(),
    E: FnOnce(String),
{
    let timeout = std::time::Duration::from_secs(timeout_secs);

    let (tx, rx) = std::sync::mpsc::channel();
    let tx_err = tx.clone();

    let ok = || {
        let _ = tx.send(Ok(()));
    };
    let err = |msg: String| {
        let _ = tx_err.send(Err(msg));
    };

    let _ = f(ok,err)?;

    tokio::time::timeout(timeout, rx)
        .await
        .map_err(|_| "timed out".to_string())?
        .ok_or("callback dropped".to_string())?
}
