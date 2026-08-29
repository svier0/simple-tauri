
/// 切换工作目录
pub fn cd(dir: &str) -> Result<(), String> {
    let path = super::to_path(dir);
    if !path.exists() {
        Err("目录不存在".to_string())
    }else if path.is_dir() {
        super::set_work_dir(&path.to_string_lossy().to_string());
        Ok(())
    }else if path.is_file() {
        Err("目标是文件".to_string())
    }else{
        Err("未知错误".to_string())
    }
}