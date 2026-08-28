
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
    let tmp_path = std::env::temp_dir().join(&fname);
    // 调用处把 Windows 反斜杠换成 /（Windows 同样识别 /），shlex 不处理 /，路径完整保留
    let tmp_file = tmp_path.to_string_lossy().replace('\\', "/");
    let dir = dir.replace('\\', "/");
    sh2rs!("wget -O {} {}", tmp_file, zipurl)?;

    // ---------- 清理目录 ----------
    sh2rs!("rm -rf {}", dir)
        .ok();
    sh2rs!("mkdir -p {}", dir)
        .map_err(|e| format!("创建目录失败: {}", e))?;

    // ---------- 解压 ----------
    sh2rs!("unzip {} {} {}", tmp_file, dir, extract_dir)
        .map_err(|e| format!("解压失败: {}", e))?;

    // ---------- 删除临时文件 ----------
    sh2rs!("rm {}", tmp_file)?;

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
/// 调用示例： let result = sync_call(|ok,err|{},1)?;
/// 提示: 当心同线程死锁(请使用async_call)
pub fn sync_call<F>(f: F,timeout_secs: u64) -> Result<(), String>
where
    F: FnOnce(Box<dyn FnOnce() + Send>, Box<dyn FnOnce(String) + Send>) -> Result<(), String>,
{
    let timeout = std::time::Duration::from_secs(timeout_secs);

    let (tx, rx) = std::sync::mpsc::channel::<Result<(), String>>();
    let tx_err = tx.clone();

    let ok: Box<dyn FnOnce() + Send> = Box::new(move || { let _ = tx.send(Ok(())); });
    let err: Box<dyn FnOnce(String) + Send> = Box::new(move |msg: String| { let _ = tx_err.send(Err(msg)); });

    f(ok, err)?;

    rx.recv_timeout(timeout)
            .map_err(|e| match e {
                std::sync::mpsc::RecvTimeoutError::Timeout => format!("callback timed out after {:?}", timeout),
                std::sync::mpsc::RecvTimeoutError::Disconnected => "callback dropped before completing".into(),
            })?
}

/// 异步事件回调转为同步阻塞（await 版）
/// 调用示例： let result = async_call(|ok,err|{},1).await?;
/// 提示: 必须在async函数中调用
pub async fn async_call<F>(f: F,timeout_secs: u64) -> Result<(), String>
where
    F: FnOnce(Box<dyn FnOnce() + Send>, Box<dyn FnOnce(String) + Send>) -> Result<(), String>,
{
    let timeout = std::time::Duration::from_secs(timeout_secs);

    let (tx, mut rx) = tokio::sync::mpsc::channel::<Result<(), String>>(1);
    let tx_err = tx.clone();

    let ok: Box<dyn FnOnce() + Send> = Box::new(move || { let _ = tx.send(Ok(())); });
    let err: Box<dyn FnOnce(String) + Send> = Box::new(move |msg: String| { let _ = tx_err.send(Err(msg)); });

    f(ok, err)?;

    tokio::time::timeout(timeout, rx.recv()).await
        .map_err(|_| format!("callback timed out after {:?}", timeout))?
        .ok_or("callback dropped before completing".to_string())?
}
