
use super::unzip_remote;

use std::env::consts::{ARCH, OS};

/// 安装node
pub fn ensure_node(ver: &str,dir: &str) -> Result<(), String> {
	if crate::simple_tray::resource_dir("server/nodejs/node.exe").is_file()
		|| crate::simple_tray::resource_dir("server/nodejs/node").is_file(){
		return Ok(());
	}
	let ver = if ver.is_empty() { "26.7.0" } else { ver };
	let dir = if dir.is_empty() { "server/nodejs" } else { dir };
	let extract_dir = format!(
		"node-v{ver}-{}-{}",
		match OS {
			"windows" => "win",
			"macos"   => "darwin",
			"linux"   => "linux",
			_ => "",
		},
		match ARCH {
			"x86_64" => "x64",
			"aarch64" => "arm64",
			_ => "",
		}
	);
	let filename = format!("{extract_dir}.{}",
		match OS {
			"windows" => "zip",
			"macos"   => "tar.gz",
			"linux"   => "tar.xz",
			_ => "",
		}
	);
	let url = format!("https://nodejs.org/dist/v{ver}/{filename}");
	// 下载并解压
    unzip_remote(&url,
    	&crate::simple_tray::resource_dir(dir).to_string_lossy(),
    	&extract_dir)?;
    Ok(())
}

/// 安装pnpm
pub fn ensure_pnpm(_ver: &str,_dir: &str) -> Result<(), String> {
	Ok(())
}