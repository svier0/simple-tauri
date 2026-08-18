/**
 * **注意**
 * 不论何种原因，此文件除了用户，禁止修改，禁止任何编辑
 */

use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};

/// 当前启动的服务进程（全局唯一）
static CHILD: OnceLock<Mutex<Option<Child>>> = OnceLock::new();
/// 上次启动的参数，restart 时复用
static LAST_ARGS: OnceLock<(String, String)> = OnceLock::new();
/// 关闭标志：置位后 wait_port 立即返回
static SHUTDOWN: AtomicBool = AtomicBool::new(false);
/// 应用资源目录（非 windows 从 app 资源目录解析用），启动时缓存
#[cfg(not(windows))]
static RESOURCE_DIR: OnceLock<PathBuf> = OnceLock::new();

/// Job Object 句柄（windows）：KILL_ON_JOB_CLOSE 下父进程退出时内核自动终止全部绑定进程
/// 以 usize 存储句柄整数值，避免 HANDLE(*mut) 的 Send/Sync 限制
#[cfg(windows)]
static JOB_HANDLE: OnceLock<usize> = OnceLock::new();

#[cfg(windows)]
use windows_sys::Win32::Foundation::HANDLE;
#[cfg(windows)]
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
    JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
};

/// 获取全局 Job Object 句柄（懒创建）
/// KILL_ON_JOB_CLOSE：句柄关闭时内核终止作业内所有进程。
/// 句柄不主动关闭，随进程退出被内核回收，从而覆盖强杀/崩溃场景。
#[cfg(windows)]
fn job_object() -> HANDLE {
    if let Some(h) = JOB_HANDLE.get() {
        return *h as HANDLE;
    }
    let name = windows_sys::core::w!("dsh-job-object");
    let handle = unsafe { CreateJobObjectW(std::ptr::null(), name) };
    if handle.is_null() {
        return std::ptr::null_mut();
    }
    let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
    info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
    let ok = unsafe {
        SetInformationJobObject(
            handle,
            windows_sys::Win32::System::JobObjects::JobObjectExtendedLimitInformation,
            &info as *const _ as *const core::ffi::c_void,
            std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        )
    };
    if ok == 0 {
        return std::ptr::null_mut();
    }
    let _ = JOB_HANDLE.set(handle as usize);
    handle
}

/// 缓存应用资源目录（启动时由 simple_tray 调用一次）
/// windows 下不缓存（按 exe 目录解析），非 windows 缓存 resource_dir
pub fn init() {
    #[cfg(not(windows))]
    {
        use tauri::Manager;
        if let Some(app) = crate::simple_tray::app() {
            if let Ok(dir) = app.path().resource_dir() {
                let _ = RESOURCE_DIR.set(dir);
            }
        }
    }
}

#[cfg(windows)]
use std::os::windows::process::CommandExt;
#[cfg(windows)]
use std::os::windows::io::AsRawHandle;

/// 隐藏新进程的 cmd 黑窗（CREATE_NO_WINDOW）
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

fn child() -> &'static Mutex<Option<Child>> {
    CHILD.get_or_init(|| Mutex::new(None))
}

/// 解析工作目录为绝对路径
/// - 绝对路径直接使用
/// - windows 相对路径只从 exe 二进制所在目录解析（不装到 C 盘/资源目录）
/// - 其它平台从 app 资源目录解析
#[cfg(windows)]
fn resolve_work_dir(work_dir: &str) -> PathBuf {
    let p = Path::new(work_dir);
    if p.is_absolute() {
        return p.to_path_buf();
    }
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    exe_dir.join(work_dir)
}

/// 解析工作目录为绝对路径（非 windows：从 app 资源目录解析）
#[cfg(not(windows))]
fn resolve_work_dir(work_dir: &str) -> PathBuf {
    let p = Path::new(work_dir);
    if p.is_absolute() {
        return p.to_path_buf();
    }
    let resource_dir = RESOURCE_DIR.get().cloned().unwrap_or_else(|| PathBuf::from("."));
    resource_dir.join(work_dir)
}

/// 启动服务器进程（挂起，不阻塞）
/// 已有进程时先停止
pub fn start(work_dir: &str, exec_cmd: &str) -> std::io::Result<()> {
    stop();
    SHUTDOWN.store(false, Ordering::SeqCst);

    let dir = resolve_work_dir(work_dir);
    let proc = spawn_in_dir(&dir, exec_cmd)?;

    *child().lock().unwrap() = Some(proc);
    let _ = LAST_ARGS.set((work_dir.to_string(), exec_cmd.to_string()));
    Ok(())
}

#[cfg(windows)]
fn spawn_in_dir(dir: &Path, exec_cmd: &str) -> std::io::Result<Child> {
    // 隐藏控制台弹框
    let mut cmd = Command::new("cmd");
    cmd.args(["/C", exec_cmd])
        .current_dir(dir)
        .creation_flags(CREATE_NO_WINDOW);
    let child = cmd.spawn()?;

    // 绑定到 Job Object：进程无法二次绑定已绑定的 job，先探测当前 job（如果已绑定则跳过）
    let handle = job_object();
    if !handle.is_null() {
        let proc_handle = child.as_raw_handle() as HANDLE;
        unsafe {
            let _ = AssignProcessToJobObject(handle, proc_handle);
        }
    }
    Ok(child)
}

#[cfg(not(windows))]
fn spawn_in_dir(dir: &Path, exec_cmd: &str) -> std::io::Result<Child> {
    Command::new("sh")
        .arg("-c")
        .arg(exec_cmd)
        .current_dir(dir)
        .spawn()
}

/// 停止服务器进程
pub fn stop() {
    SHUTDOWN.store(true, Ordering::SeqCst);
    if let Some(mut child) = child().lock().unwrap().take() {
        let _ = child.kill();
        let _ = child.wait();
    }
}

/// 重启服务器进程（复用上次 start 的参数）
pub fn restart() -> std::io::Result<()> {
    SHUTDOWN.store(false, Ordering::SeqCst);
    match LAST_ARGS.get() {
        Some((dir, cmd)) => start(dir, cmd),
        None => Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "尚未调用过 start，无法 restart",
        )),
    }
}

/// 等待指定端口可以连接（服务启动成功标志）
/// 最多等待 timeout（秒），期间每隔 interval（毫秒）探测一次
/// 若服务器进程已提前退出，立即返回错误（不用等满超时）
pub fn wait_port(port: u16) -> std::io::Result<()> {
    wait_port_timeout(port, 15, 300)
}

/// 等待指定端口可以连接，自定义超时/间隔
pub fn wait_port_timeout(port: u16, timeout_secs: u64, interval_ms: u64) -> std::io::Result<()> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(timeout_secs);
    loop {
        if SHUTDOWN.load(Ordering::SeqCst) {
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
        std::thread::sleep(std::time::Duration::from_millis(interval_ms));
    }
}