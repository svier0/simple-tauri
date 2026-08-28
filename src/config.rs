
use serde_json::Value;

/// 配置文件路径（exe目录或资源目录下/config.json，绝对路径）。
pub const CONFIG_FILE: &str = "config.json";

/// 默认配置模板。文件不存在时生成此内容。
const DEFAULT_CONFIG: &str = r#"
{
	"auto_start":false,
	"auto_run":false,
	"silent_launch":false,
	"auto_update":false,
}
"#;

/// 加载配置。文件不存在时生成默认配置文件；存在但解析失败则报错。
pub fn load() -> Result<()> {
    let path = crate::simple_tray::resource_dir(CONFIG_FILE);
    if !path.exists() {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("创建配置目录 {} 失败", parent.display()))?;
        }
        std::fs::write(&path, DEFAULT_CONFIG)
            .with_context(|| format!("生成配置文件 {} 失败", path.display()))?;
        info!("配置文件不存在，已生成默认配置 {}", path.display());
        return Ok(());
    }
    from_path(&path)
        .with_context(|| format!("解析配置文件 {} 失败", path.display()))?;
    info!("已加载配置文件 {}", path.display());
    Ok(())
}

/// 读取配置
pub fn get_config(){
	//
}

/// 修改配置
pub fn set_config(){
	//
}
