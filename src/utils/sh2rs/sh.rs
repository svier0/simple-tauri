
/// 启动脚本
pub fn sh(script: &str) -> Result<(), String> {
    #[cfg(windows)]
    use std::os::windows::process::CommandExt;

    let dir = super::to_path("");

    #[cfg(windows)]
    let out = std::process::Command::new("cmd")
        .args(["/C", script])
        .current_dir(dir)
        .creation_flags(0x0800_0000)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .expect("failed to execute");
    #[cfg(not(windows))]
    let out = std::process::Command::new("sh")
        .arg("-c")
        .arg(script)
        .current_dir(dir)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .expect("failed to execute");

    // 调试输出 始终为空
    eprintln!("{}", String::from_utf8_lossy(&out.stdout));

    super::set_stdout_buffer(&format!("{}",String::from_utf8_lossy(&out.stdout)));

    let err = String::from_utf8_lossy(&out.stderr);
    if !err.is_empty() {
        return Err(err.to_string());
    }

    if !out.status.success() {
        let code = out.status.code()
            .map(|n| format!("exit code {n}"))
            .unwrap_or_else(|| "terminated by signal".to_string());

        let stderr = String::from_utf8_lossy(&out.stderr);
        return Err(format!("{code}\nstderr:\n{stderr}"));
    }

    Ok(())
}
