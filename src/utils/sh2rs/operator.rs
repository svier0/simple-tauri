use std::sync::Mutex;

/// 输出缓冲区
static STDOUT_BUFFER: Mutex<Option<String>> = Mutex::new(None);
/// 错误缓冲区
static ERROR_BUFFER: Mutex<Option<String>> = Mutex::new(None);

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
pub fn set_error_buffer(buffer: &str) {
    *ERROR_BUFFER.lock().unwrap() = Some(buffer.to_string());
}

/// 获取错误缓冲区
pub fn get_error_buffer() -> String {
    let r = ERROR_BUFFER
        .lock()
        .unwrap()
        .clone()
        .unwrap_or_default();
    *ERROR_BUFFER.lock().unwrap() = None;
    r
}

/// 输入重定向符号
pub fn in_redirection(token: &str,origin: &str) -> Result<(), String> {
	Err("未实现的操作符".to_string())
}

/// 输出重定向符号
pub fn out_redirection(token: &str,target: &str) -> Result<(), String> {
	Err("未实现的操作符".to_string())
    // target为文件 get_stdout_buffer()写入到target
}

/// 追加重定向符号
pub fn append_redirection(token: &str,target: &str) -> Result<(), String> {
	Err("未实现的操作符".to_string())
    // target为文件 get_stdout_buffer()追加写入到target
}

/// 错误重定向符号
pub fn error_redirection(token: &str,target: &str) -> Result<(), String> {
    Err("未实现的操作符".to_string())
    // target为文件 get_stdout_buffer()追加写入到target
}