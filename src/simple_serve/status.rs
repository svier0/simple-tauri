
use super::action::{child};

use std::sync::atomic::{AtomicBool, Ordering};

/// 任意数字/字符串 → u16 转换 trait
pub trait IntoPort {
    fn into_port(self) -> u16;
}

impl IntoPort for u16 {
    fn into_port(self) -> u16 { self }
}
impl IntoPort for i32 {
    fn into_port(self) -> u16 { self as u16 }
}
impl IntoPort for u32 {
    fn into_port(self) -> u16 { self as u16 }
}
impl IntoPort for i64 {
    fn into_port(self) -> u16 { self as u16 }
}
impl IntoPort for u64 {
    fn into_port(self) -> u16 { self as u16 }
}
impl IntoPort for &str {
    fn into_port(self) -> u16 { self.parse().unwrap_or(0) }
}
impl IntoPort for String {
    fn into_port(self) -> u16 { self.parse().unwrap_or(0) }
}

/// 运行状态
static RUNNING: AtomicBool = AtomicBool::new(false);

/// 返回当前服务运行状态
pub fn is_running() -> bool {
	RUNNING.load(Ordering::SeqCst)
}

/// 设置当前服务运行状态
pub(super) fn set_run_flag(flag: bool){
	RUNNING.store(flag, Ordering::SeqCst);
}

/// 等待指定端口可以连接（服务启动成功标志）
/// 最多等待 timeout（秒），期间每隔 interval（秒）探测一次
/// 若服务器进程已提前退出，立即返回错误（不用等满超时）
pub fn wait_port(port: impl IntoPort) -> std::io::Result<()> {
    wait_port_timeout(port, 15, 0.3)
}

/// 等待指定端口可以连接，自定义超时/间隔
pub fn wait_port_timeout(port: impl IntoPort, timeout_secs: u64, interval: f64) -> std::io::Result<()> {
    let port = port.into_port();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(timeout_secs);
    loop {
        if !is_running() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Interrupted,
                "服务器已停止",
            ));
        }
        if std::net::TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return Ok(());
        }
        // 进程已退出（启动失败），立即报错，避免干等超时
        let proc = child().lock().unwrap().take();
        if let Some(mut proc) = proc {
            match proc.try_wait() {
                Ok(Some(status)) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("服务器进程已退出，退出码: {status}"),
                    ));
                }
                Ok(None) => {
                    // 进程还活着，放回去继续等
                    *child().lock().unwrap() = Some(proc);
                }
                Err(e) => return Err(e),
            }
        }
        if std::time::Instant::now() >= deadline {
            return Err(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                format!("端口 {port} 等待连接超时"),
            ));
        }
        let _ = crate::utils::sh2rs::sh2rs!("sleep {}",interval);
    }
}
