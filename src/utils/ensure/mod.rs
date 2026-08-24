
use super::unzip_remote;

use std::env::consts::{ARCH, OS};

/// 安装node
pub fn ensure_node(ver: &str,dir: &str) -> Result<(), String> {
	let ver = if ver.is_empty() { "26.7.0" } else { ver };
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
    	crate::simple_tray::resource_dir("server/nodejs"),
    	extract_dir)?;
    Ok(())
}