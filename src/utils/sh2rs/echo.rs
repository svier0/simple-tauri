
/// 输出文本到输出缓冲区
pub fn echo(text: &str) -> Result<(), String> {
    super::set_stdout_buffer(&format!("{text}\n"));
    Ok(())
}