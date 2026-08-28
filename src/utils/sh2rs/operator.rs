use std::sync::Mutex;

/// 输出缓冲区
static STDOUT_BUFFER: Mutex<Option<String>> = Mutex::new(None);

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

/// 输入重定向符号
pub fn in_redirection() -> Result<(), String> {
	Err("未实现的操作符".to_string())
}

/// 输出重定向符号
pub fn out_redirection() -> Result<(), String> {
	Err("未实现的操作符".to_string())
}

/// 追加重定向符号
pub fn append_redirection() -> Result<(), String> {
	Err("未实现的操作符".to_string())
}