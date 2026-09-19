
use crate::utils::sh2rs::sh2rs;
use crate::utils::sh2rs::try_quote;
use indoc::indoc;

use std::env::consts::{ARCH, OS};
use std::sync::OnceLock;

static NODE_DIR: OnceLock<String> = OnceLock::new();
static PY_DIR:   OnceLock<String> = OnceLock::new();

/// 安装node
pub fn ensure_node(ver: &str,node_dir: &str) -> Result<(), String> {
	let node_dir = if node_dir.is_empty() { "server/nodejs" } else { node_dir };

	let node_path = super::path_rel2abs(node_dir,crate::simple_tray::resource_dir(""));
    let node_dir = node_path.to_string_lossy().replace("\\","/");
	let _ = NODE_DIR.set(node_dir.clone());

	if node_path.join("node.exe").is_file()
		|| node_path.join("node").is_file() {
		return Ok(());
	}

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
    sh2rs!("cd {}",crate::simple_tray::resource_dir("").to_string_lossy().replace("\\","/"))?;
    super::unzip_remote(&url,&node_dir,&extract_dir)?;
    sh2rs!("mkdir -p {}/bin",node_dir).ok();
    sh2rs!("mkdir -p {}/cache",node_dir).ok();
    sh2rs!("echo {} > {}"
    	,try_quote!("cache=\"{}/cache\"\nprefix=\"{}/bin\"\n{}\n{}\n{}",
    		node_dir,
    		node_dir,
    		"registry = \"https://registry.npmmirror.com\"",
    		"loglevel=error",
    		"enabled-https-notices=false")
    	,format!("{node_dir}/node_modules/npm/.npmrc")).ok();
    sh2rs!("cd -").ok();
    Ok(())
}

/// 安装pnpm
pub fn ensure_pnpm(_ver: &str) -> Result<(), String> {
	let node_dir = NODE_DIR.get_or_init(||"".to_string());

	let node_path = std::path::Path::new(&node_dir);
	if node_path.join("bin/pnpm").is_file() {
		return Ok(());
	}

	#[cfg(windows)]
	run_env_cmd("call npm install pnpm -g")?;
	#[cfg(not(windows))]
	run_env_cmd("npm install pnpm -g")?;

	if node_path.join("bin/pnpm").is_file() {
		return Ok(());
	}
	Err("安装失败".to_string())
}

/// 安装python
pub fn ensure_python(ver: &str,py_dir: &str) -> Result<(), String> {
	let py_dir = if py_dir.is_empty() { "server/nodejs" } else { py_dir };

	let py_path = super::path_rel2abs(py_dir,crate::simple_tray::resource_dir(""));
    let py_dir = py_path.to_string_lossy().replace("\\","/");
	let _ = PY_DIR.set(py_dir.clone());

	if py_path.join("node.exe").is_file()
		|| py_path.join("node").is_file() {
		return Ok(());
	}

	let ver = if ver.is_empty() { "3.13.14" } else { ver };
	let extract_dir = "";
	#[cfg(windows)]
	#[cfg(target_arch = "x86_64")]
	let url = format!("https://www.python.org/ftp/python/{ver}/python-{ver}-embed-amd64.zip");
	#[cfg(windows)]
	#[cfg(target_arch = "aarch64")]
	let url = format!("https://www.python.org/ftp/python/3.14.7/python-3.14.7-embed-arm64.zip");
	#[cfg(target_os = "macos")]
	let url = format!("https://www.python.org/ftp/python/3.14.7/python-3.14.7-macos11.pkg");
	// 下载并解压
    sh2rs!("cd {}",crate::simple_tray::resource_dir("").to_string_lossy().replace("\\","/"))?;
    super::unzip_remote(&url,&py_dir,&extract_dir)?;
    sh2rs!("cd -").ok();
    Ok(())
}

/// 获取带环境变量的命令行代码
pub fn get_env_cmd(cmd:&str) -> String {
	let node_dir = NODE_DIR.get_or_init(||"".to_string());
	let py_dir   = PY_DIR.get_or_init(||"".to_string());
	#[cfg(windows)]
	return format!(indoc! {r#"
			@echo off{}{}
			set "npm_config_userconfig=%NODE_HOME%/node_modules/npm/.npmrc"
			set "PATH=%NODE_HOME%;%NODE_HOME%/bin;%PYTHON_HOME%;"
			{}
		"#},
		if node_dir.is_empty() { "".to_string() }else{ format!("\nset \"NODE_HOME={node_dir}\"") },
		if py_dir.is_empty() {   "".to_string() }else{ format!("\nset \"PYTHON_HOME={py_dir}\"") },
		cmd);
	#[cfg(not(windows))]
	return format!(indoc! {r#"
			#!/bin/bash{}{}
			npm_config_userconfig=$NODE_HOME/node_modules/npm/.npmrc
			PATH=$NODE_HOME:$NODE_HOME/bin:$PYTHON_HOME"
			{}
		"#},
		if node_dir.is_empty() { "".to_string() }else{ format!("NODE_HOME={node_dir}\n") },
		if py_dir.is_empty() {   "".to_string() }else{ format!("PYTHON_HOME={py_dir}\n") },
		cmd);
}

/// 带环境变量执行命令
pub fn run_env_cmd(cmd:&str) -> Result<(), String> {
	let cmd = simple_tauri::utils::get_env_cmd(cmd);
	sh2rs!("sh {}",try_quote!("{}",cmd))?;
	Ok(())
}

/// 写入带环境变量的脚本
pub fn write_env_script(cmd:&str,file:&str) -> Result<(), String> {
	let cmd = simple_tauri::utils::get_env_cmd(cmd);
	sh2rs!("echo {} > {}",try_quote!("{}", cmd),file)?;
	Ok(())
}

/// 弹出命令行窗口
pub fn show_env_cmd(cmd:&str) {
	let cmd = simple_tauri::utils::get_env_cmd(cmd);

    // 启动cmd窗口并执行字符串变量cmd的代码 然后cmd窗口停留等待用户输入
    #[cfg(windows)]
    {
        let tmp = std::env::temp_dir().join("_simple_tauri_cmd.cmd");
        std::fs::write(&tmp, format!("{cmd}\r\npause")).ok();
        std::process::Command::new("cmd")
            .args(["/C", "start", "cmd", "/K", tmp.to_string_lossy().as_ref()])
            .spawn().ok();
    }
    #[cfg(not(windows))]
    std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .spawn().ok();
}