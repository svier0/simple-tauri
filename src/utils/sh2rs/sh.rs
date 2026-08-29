
/// 启动脚本
pub fn sh(script: &str) -> Result<(), String> {
    let child = spawn_in_dir(super::to_path(""),script)
        .map_err(|e|format!("{}",e))?;
    super::set_stdout_buffer(&format!("PID: {}",child.id()));
    Ok(())
}

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
fn spawn_in_dir(dir: std::path::PathBuf, exec_cmd: &str) -> std::io::Result<std::process::Child> {
    std::process::Command::new("cmd")
        .args(["/C", exec_cmd])
        .current_dir(dir)
        .creation_flags(0x0800_0000)
        .spawn()
}

#[cfg(not(windows))]
fn spawn_in_dir(dir: std::path::PathBuf, exec_cmd: &str) -> std::io::Result<std::process::Child> {
    std::process::Command::new("sh")
        .arg("-c")
        .arg(exec_cmd)
        .current_dir(dir)
        .spawn()
}