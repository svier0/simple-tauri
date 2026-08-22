use std::fs;
use std::path::Path;

/// 解压zip文件 至指定目录 (可选提取目录，默认提取全部)
pub fn unzip(zipfile: &str,dir: &str,extract_dir: &str) -> Result<(), String> {
	let zip_file = fs::File::open(zipfile)
        .map_err(|e| format!("打开 zip 失败: {}", e))?;
    let mut archive = zip::ZipArchive::new(zip_file)
        .map_err(|e| format!("读取 zip 失败: {}", e))?;

    let extract_dir = if !extract_dir.is_empty() {
        let cleaned = extract_dir
            .trim_start_matches('/')
            .trim_end_matches('/')
            .to_string();
        format!("{}/", cleaned)
    } else {
        String::new()
    };

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)
            .map_err(|e| format!("读取条目失败: {}", e))?;
        let name = entry.name().to_string();

        if !extract_dir.is_empty() && !name.starts_with(&extract_dir) {
            continue;
        }

        let relative = if !extract_dir.is_empty() {
            name.strip_prefix(&extract_dir).unwrap_or(&name)
        } else {
            &name
        };

        let out_path = Path::new(dir).join(relative);

        if name.ends_with('/') {
            fs::create_dir_all(&out_path)
                .map_err(|e| format!("创建目录 {} 失败: {}", name, e))?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("创建父目录失败: {}", e))?;
            }
            let mut out = fs::File::create(&out_path)
                .map_err(|e| format!("创建文件 {} 失败: {}", name, e))?;
            std::io::copy(&mut entry, &mut out)
                .map_err(|e| format!("写入 {} 失败: {}", name, e))?;
        }
    }
    Ok(())
}