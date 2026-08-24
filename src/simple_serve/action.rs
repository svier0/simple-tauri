
use super::status::{set_run_flag};
use super::work_dir::{get_work_dir};
use super::spawn::{spawn_in_dir};

use std::process::{Child};
use std::sync::{Mutex, OnceLock};

/// 服务器启动命令
static SERVER_START_CMD: Mutex<Option<String>> = Mutex::new(None);
/// 当前启动的服务进程（全局唯一）
static CHILD: OnceLock<Mutex<Option<Child>>> = OnceLock::new();

pub fn child() -> &'static Mutex<Option<Child>> {
    CHILD.get_or_init(|| Mutex::new(None))
}

/// 获取设置的服务器启动命令
fn get_start_cmd() -> String {
    SERVER_START_CMD.lock().unwrap().clone().unwrap_or_default()
}

/// 设置服务器启动命令
pub fn set_start_cmd(cmd: &str) {
    *SERVER_START_CMD.lock().unwrap() = Some(cmd.to_string());
}

/// 启动服务器进程（挂起，不阻塞）
/// 已有进程时先停止
pub fn start() -> std::io::Result<()> {
    stop();
    set_run_flag(true);

    let dir = get_work_dir(None);
    let proc = spawn_in_dir(std::path::Path::new(&dir), &get_start_cmd())?;

    *child().lock().unwrap() = Some(proc);
    Ok(())
}

/// 停止服务器进程
pub fn stop() {
    set_run_flag(false);
    if let Some(mut child) = child().lock().unwrap().take() {
        let _ = child.kill();
        let _ = child.wait();
    }
}