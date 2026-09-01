
/// 启动脚本
pub fn sh(script: &str) -> Result<(), String> {
    #[cfg(windows)]
    use std::os::windows::process::CommandExt;

    let dir = super::to_path("");

    let out = match std::env::consts::OS {
        "windows"=>{
            let mut bat_script = String::new();
            if script.contains("\n") {
                let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
                let tmp_file = std::env::temp_dir().join(format!("tmp_sh2rs_sh_{}.bat", ts));
                bat_script = tmp_file.to_string_lossy().to_string().replace('\\', "/");
                super::sh2rs!("echo {} > {}",super::try_quote!("{}",script),bat_script)?;
            }
            let r = std::process::Command::new("cmd")
                .args(["/C", if bat_script.is_empty() {script}else{&bat_script}])
                .current_dir(dir)
                .creation_flags(0x0800_0000)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .output()
                .expect("failed to execute");
            if !bat_script.is_empty() { super::sh2rs!("rm {}",bat_script).ok(); }
            r
        }
        _ => {
            std::process::Command::new("sh")
                .arg("-c")
                .arg(script)
                .current_dir(dir)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .output()
                .expect("failed to execute")
        }
    };

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
