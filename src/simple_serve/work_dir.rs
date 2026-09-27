use std::sync::{OnceLock};

/// 服务器工作目录规则（含 <ver> 占位符，OnceLock 只设一次）
static WORK_DIR: OnceLock<String> = OnceLock::new();

/// 设置服务器工作目录规则（含 <ver> 占位符，只设一次）
pub fn set_work_dir(rule: &str) {
    let _ = WORK_DIR.set(rule.to_string());
}

/// 获取工作目录规则
/// ver 为 Some 时替换 <ver> 并返回绝对路径；为 None 时返回原始 rule（保留 <ver> 占位符）
/// 读取全局 WORK_DIR（由 set_work_dir 设置）；未调用 set_work_dir 会使用默认值"server/v<ver>"
pub fn get_work_dir(ver: Option<&str>) -> String {
    let rule = WORK_DIR
        .get_or_init(||"server/v<ver>".to_string());
    let r = match ver {
        None => rule.clone(),
        Some(v) => rule.replace("<ver>", v),
    };
    crate::simple_tray::resource_dir(&r)
        .to_string_lossy()
        .into_owned()
        .replace(std::path::MAIN_SEPARATOR_STR,"/")
}