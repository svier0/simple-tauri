
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::OnceLock;
use std::fs::File;

#[cfg(windows)]
use windows_sys::Win32::Foundation::HANDLE;
#[cfg(windows)]
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
    JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
};
#[cfg(windows)]
use std::os::windows::process::CommandExt;
#[cfg(windows)]
use std::os::windows::io::AsRawHandle;

/// Job Object 句柄（windows）：KILL_ON_JOB_CLOSE 下父进程退出时内核自动终止全部绑定进程
/// 以 usize 存储句柄整数值，避免 HANDLE(*mut) 的 Send/Sync 限制
#[cfg(windows)]
static JOB_HANDLE: OnceLock<usize> = OnceLock::new();

/// 获取全局 Job Object 句柄（懒创建）
/// KILL_ON_JOB_CLOSE：句柄关闭时内核终止作业内所有进程。
/// 句柄不主动关闭，随进程退出被内核回收，从而覆盖强杀/崩溃场景。
#[cfg(windows)]
fn job_object() -> HANDLE {
    if let Some(h) = JOB_HANDLE.get() {
        return *h as HANDLE;
    }
    let handle = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
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

/// 隐藏新进程的 cmd 黑窗（CREATE_NO_WINDOW）
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[cfg(windows)]
pub fn spawn_in_dir(dir: &Path, exec_cmd: &str) -> std::io::Result<Child> {
    let log_path = super::get_log_path();
    let log_file = File::create(&log_path)?;

    // 隐藏控制台弹框
    let mut cmd = Command::new("cmd");
    cmd.args(["/C", exec_cmd])
        .current_dir(dir)
        .creation_flags(CREATE_NO_WINDOW)
        .stdout(Stdio::from(log_file.try_clone()?))
        .stderr(Stdio::from(log_file));
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
pub fn spawn_in_dir(dir: &Path, exec_cmd: &str) -> std::io::Result<Child> {
    let log_path = super::get_log_path();
    let log_file = File::create(&log_path)?;

    Command::new("sh")
        .arg("-c")
        .arg(exec_cmd)
        .current_dir(dir)
        .stdout(Stdio::from(log_file.try_clone()?))
        .stderr(Stdio::from(log_file))
        .spawn()
}