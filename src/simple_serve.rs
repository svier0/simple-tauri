/**
 * **注意**
 * 不论何种原因，此文件除了用户，禁止修改，禁止任何编辑
 */

use std::path::Path;
use std::process::{Child, Command};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};

/// 当前启动的服务进程（全局唯一）
static CHILD: OnceLock<Mutex<Option<Child>>> = OnceLock::new();
/// 服务器工作目录规则（含 <ver> 占位符，OnceLock 只设一次）
static WORK_DIR: OnceLock<String> = OnceLock::new();
/// 本地版本号（set_local_ver 设置后替换 <ver>）
static LOCAL_VER: Mutex<Option<String>> = Mutex::new(None);
/// 上次启动的参数，restart 时复用
static LAST_ARGS: OnceLock<String> = OnceLock::new();
/// 关闭标志：置位后 wait_port 立即返回
static SHUTDOWN: AtomicBool = AtomicBool::new(false);

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

/// 设置服务器工作目录规则（含 <ver> 占位符，只设一次）
pub fn set_work_dir(rule: &str) {
    let _ = WORK_DIR.set(rule.to_string());
}

/// 设置本地版本号（用于替换 work_dir 中的 <ver>）
pub fn set_local_ver(ver: &str) {
    *LOCAL_VER.lock().unwrap() = Some(ver.to_string());
}

/// 获取设置的本地版本号
pub fn get_local_ver() -> String {
    LOCAL_VER.lock().unwrap().clone().unwrap_or_default()
}

/// 获取工作目录的绝对路径
/// work_dir.replace("<ver>", ver)，ver 为 None 时使用 LOCAL_VER
pub fn get_work_dir(ver: Option<&str>) -> Option<std::path::PathBuf> {
    let rule = WORK_DIR.get()?;
    let v = ver.map(|s| s.to_string())
        .or_else(|| LOCAL_VER.lock().unwrap().clone())
        .unwrap_or_default();
    Some(crate::simple_tray::resource_dir(&rule.replace("<ver>", &v)))
}

/// 启动服务器进程（挂起，不阻塞）
/// 已有进程时先停止
pub fn start(exec_cmd: &str) -> std::io::Result<()> {
    stop();
    SHUTDOWN.store(false, Ordering::SeqCst);

    let dir = get_work_dir(None)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "未调用 set_work_dir"))?;
    let proc = spawn_in_dir(&dir, exec_cmd)?;

    *child().lock().unwrap() = Some(proc);
    let _ = LAST_ARGS.set(exec_cmd.to_string());
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
        Some(cmd) => start(cmd),
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

/// 检测本地版本号：用 WORK_DIR 规则扫描目录，返回匹配的版本号；没有返回空字符串
pub fn check_local_ver() -> String {
    let rule = match WORK_DIR.get() {
        Some(r) => r.clone(),
        None => return String::new(),
    };
    let p = Path::new(&rule);
    // 1. 相对路径拼接resourcedir转绝对路径
    let abs = if p.is_absolute() {
        p.to_path_buf()
    } else {
        crate::simple_tray::resource_dir(&rule)
    };
    // 2. 绝对路径取父目录
    let parent = abs.parent().unwrap_or(Path::new("/"));
    // 3. 遍历父目录下所有目录，寻找符合 rule 格式的
    let fname = match p.file_name().and_then(|n| n.to_str()) {
        Some(f) => f,
        None => return String::new(),
    };
    let ver_pos = match fname.find("<ver>") {
        Some(pos) => pos,
        None => return String::new(),
    };
    let prefix = &fname[..ver_pos];
    let suffix = &fname[ver_pos + 5..];
    let entries = match std::fs::read_dir(parent) {
        Ok(e) => e,
        Err(_) => return String::new(),
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !name.starts_with(prefix) {
            continue;
        }
        let rest = &name[prefix.len()..];
        if let Some(ver) = rest.strip_suffix(suffix) {
            if !ver.is_empty() && entry.path().is_dir() {
                return ver.to_string();
            }
        }
    }
    String::new()
}