use std::io::Write;
use std::path::Path;
use std::sync::Mutex;

/// 输入缓冲区
static STDIN_BUFFER: Mutex<Option<String>> = Mutex::new(None);
/// 输出缓冲区
static STDOUT_BUFFER: Mutex<Option<String>> = Mutex::new(None);
/// 错误缓冲区
static STDERR_BUFFER: Mutex<Option<String>> = Mutex::new(None);
/// 工作目录
static WORK_DIR: Mutex<Option<String>> = Mutex::new(None);

/// 设置输入缓冲区
pub fn set_stdin_buffer(buffer: &str) {
    *STDIN_BUFFER.lock().unwrap() = Some(buffer.to_string());
}

/// 获取输入缓冲区
pub fn get_stdin_buffer() -> String {
    let r = STDIN_BUFFER
        .lock()
        .unwrap()
        .clone()
        .unwrap_or_default();
    *STDIN_BUFFER.lock().unwrap() = None;
    r
}

/// 设置输出缓冲区
pub fn set_stdout_buffer(buffer: &str) {
    *STDOUT_BUFFER.lock().unwrap() = Some(buffer.to_string());
}

/// 获取输出缓冲区
pub fn get_stdout_buffer() -> String {
    let r = STDOUT_BUFFER
        .lock()
        .unwrap()
        .clone()
        .unwrap_or_default();
    *STDOUT_BUFFER.lock().unwrap() = None;
    r
}

/// 设置错误缓冲区
pub fn set_stderr_buffer(buffer: &str) {
    *STDERR_BUFFER.lock().unwrap() = Some(buffer.to_string());
}

/// 获取错误缓冲区
pub fn get_stderr_buffer() -> String {
    let r = STDERR_BUFFER
        .lock()
        .unwrap()
        .clone()
        .unwrap_or_default();
    *STDERR_BUFFER.lock().unwrap() = None;
    r
}

/// 设置工作目录
pub fn set_work_dir(dir: &str) {
    *WORK_DIR.lock().unwrap() = Some(dir.to_string());
}

/// 获取工作目录
pub fn get_work_dir() -> String {
    WORK_DIR
        .lock()
        .unwrap()
        .clone()
        .unwrap_or(crate::simple_tray::resource_dir("").to_string_lossy().to_string())
}

/// 相对路径转绝对路径
pub(crate) fn to_path(target: &str) -> std::path::PathBuf {
    let p = Path::new(target);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        Path::new(&get_work_dir()).join(target)
    }
}

/// 判断 target 是否为可写的普通文件路径：
/// - 已存在的普通文件 → 可写（覆盖）
/// - 不存在但父目录存在（或当前目录下的新文件名）→ 可新建
/// - 目录或其它非法路径 → 否（不应被当作文件写入）
fn is_regular_file_path(target: &str) -> bool {
    let p = super::to_path(target);
    if p.is_dir() {
        return false;
    }
    if p.is_file() {
        return true;
    }
    match p.parent() {
        Some(parent) if parent.as_os_str().is_empty() => true,
        Some(parent) => parent.is_dir(),
        None => true,
    }
}

/// 判断 target 是否为特殊文件系统/设备路径：不允许当作普通文件任意读写
/// Unix: /dev、/proc、/sys；Windows: NUL、CON、AUX、PRN、COM*、LPT*
fn is_special_fs_path(target: &str) -> bool {
    if target.starts_with("/dev/") || target.starts_with("/proc/") || target.starts_with("/sys/") {
        return true;
    }
    let u = target.to_ascii_uppercase();
    if matches!(u.as_str(), "NUL" | "CON" | "AUX" | "PRN") {
        return true;
    }
    if (u.starts_with("COM") || u.starts_with("LPT"))
        && u.len() == 4
        && u[3..].chars().all(|c| c.is_ascii_digit())
    {
        return true;
    }
    false
}

/// 输入重定向符号
/// 穷举可跟类型：普通文件路径 / &N 文件描述符 / /dev/null / <(...) 进程替换 / 终端
pub fn in_redirection(_token: &str, origin: &str) -> Result<(), String> {
    if origin.starts_with('&') || origin.starts_with("<(") {
        // &N 文件描述符、<(...) 进程替换：未实现
        Err("未实现的操作符".to_string())
    } else if origin == "/dev/null" {
        // 丢弃输入
        set_stdin_buffer("");
        Ok(())
    } else if is_special_fs_path(origin) {
        // /dev、/proc、/sys、Windows 设备等特殊文件系统：不允许任意读写，未实现
        Err("未实现的操作符".to_string())
    } else if Path::new(origin).is_file() {
        // 普通文件：读入 stdin 缓冲
        let content = std::fs::read_to_string(origin).map_err(|e| e.to_string())?;
        set_stdin_buffer(&content);
        Ok(())
    } else {
        // 其它未穷举到的类型 → 未实现
        Err("未实现的操作符".to_string())
    }
}

/// 输出重定向符号
/// 穷举可跟类型：普通文件路径（含 /dev、/proc、/sys 下文件）/ &N 文件描述符 / /dev/null / >(...) 进程替换（后两者未实现）
pub fn out_redirection(_token: &str, target: &str) -> Result<(), String> {
    if target.starts_with('&') || target.starts_with(">(") {
        // &N 文件描述符、>(...) 进程替换：未实现
        Err("未实现的操作符".to_string())
    } else if target == "/dev/null" || target.eq_ignore_ascii_case("NUL") {
        // 丢弃标准输出
        get_stdout_buffer();
        Ok(())
    } else if is_special_fs_path(target) {
        // /dev、/proc、/sys、Windows 设备等特殊文件系统：不允许任意读写，未实现
        Err("未实现的操作符".to_string())
    } else if is_regular_file_path(target) {
        // 普通文件路径：覆盖或新建
        std::fs::write(target, get_stdout_buffer()).map_err(|e| e.to_string())
    } else {
        // 其它未穷举到的类型（目录、非法路径等）→ 未实现
        Err("未实现的操作符".to_string())
    }
}

/// 追加重定向符号
/// 穷举可跟类型：普通文件路径（含 /dev、/proc、/sys 下文件）/ &N 文件描述符 / /dev/null / >(...) 进程替换（后两者未实现）
pub fn append_redirection(_token: &str, target: &str) -> Result<(), String> {
    if target.starts_with('&') || target.starts_with(">(") {
        // &N 文件描述符、>(...) 进程替换：未实现
        Err("未实现的操作符".to_string())
    } else if target == "/dev/null" || target.eq_ignore_ascii_case("NUL") {
        // 丢弃标准输出
        get_stdout_buffer();
        Ok(())
    } else if is_special_fs_path(target) {
        // /dev、/proc、/sys、Windows 设备等特殊文件系统：不允许任意读写，未实现
        Err("未实现的操作符".to_string())
    } else if is_regular_file_path(target) {
        // 普通文件路径：追加或新建
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(target)
            .map_err(|e| e.to_string())?;
        f.write_all(get_stdout_buffer().as_bytes())
            .map_err(|e| e.to_string())
    } else {
        // 其它未穷举到的类型 → 未实现
        Err("未实现的操作符".to_string())
    }
}

/// 错误重定向符号
/// 穷举可跟类型：普通文件路径（含 /dev、/proc、/sys 下文件）/ &N 文件描述符 / /dev/null / >(...) 进程替换（后两者未实现）
pub fn error_redirection(_token: &str, target: &str) -> Result<(), String> {
    if target.starts_with('&') || target.starts_with(">(") {
        // &N 文件描述符、>(...) 进程替换：未实现
        Err("未实现的操作符".to_string())
    } else if target == "/dev/null" || target.eq_ignore_ascii_case("NUL") {
        // 丢弃标准错误
        get_stderr_buffer();
        Ok(())
    } else if is_special_fs_path(target) {
        // /dev、/proc、/sys、Windows 设备等特殊文件系统：不允许任意读写，未实现
        Err("未实现的操作符".to_string())
    } else if is_regular_file_path(target) {
        // 普通文件路径：覆盖或新建
        std::fs::write(target, get_stderr_buffer()).map_err(|e| e.to_string())
    } else {
        // 其它未穷举到的类型 → 未实现
        Err("未实现的操作符".to_string())
    }
}