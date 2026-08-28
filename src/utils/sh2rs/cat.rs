
/// 读取文本到输出缓冲区
pub fn cat(filepath: &str) -> Result<(), String> {
    let text = std::fs::read_to_string(filepath).map_err(|e| e.to_string())?;
    super::set_stdout_buffer(&text);
    Ok(())
}