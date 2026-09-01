
use crate::utils::sh2rs::sh2rs;
use crate::utils::sh2rs::try_quote;
use indoc::indoc;

use std::env::consts::{ARCH, OS};

/// 安装node
pub fn ensure_node(ver: &str,node_dir: &str) -> Result<(), String> {
	let node_dir = if node_dir.is_empty() { "server/nodejs" } else { node_dir };
	let ver = if ver.is_empty() { "26.7.0" } else { ver };

	if crate::simple_tray::resource_dir(&format!("{node_dir}/node.exe")).is_file()
		|| crate::simple_tray::resource_dir(&format!("{node_dir}/node")).is_file(){
		return Ok(());
	}
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
    super::unzip_remote(&url,node_dir,&extract_dir)?;
    sh2rs!("mkdir -p {}/bin",node_dir).ok();
    sh2rs!("mkdir -p {}/cache",node_dir).ok();
    let node_dir = crate::simple_tray::resource_dir(node_dir).to_string_lossy().replace("\\","/");
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
pub fn ensure_pnpm(_ver: &str,node_dir: &str) -> Result<(), String> {
	let node_dir = if node_dir.is_empty() { "server/nodejs" } else { node_dir };

	if crate::simple_tray::resource_dir(&format!("{node_dir}/bin/pnpm")).is_file(){
		return Ok(());
	}

	#[cfg(windows)]
	let cmd = get_node_cmd("call npm install pnpm -g",node_dir);
	#[cfg(not(windows))]
	let cmd = get_node_cmd("npm install pnpm -g",node_dir);

	sh2rs!("sh {}",try_quote!("{}",cmd))?;
	if crate::simple_tray::resource_dir(&format!("{node_dir}/bin/pnpm")).is_file(){
		return Ok(());
	}
	Err("安装失败".to_string())
}

/// 获取node环境变量
pub fn get_node_cmd(cmd:&str,node_dir: &str) -> String {
	let node_dir = if node_dir.is_empty() { "server/nodejs" } else { node_dir };
	#[cfg(windows)]
	let cmd_pre = indoc! {r#"
		@echo off
		set "NODE_HOME=<NODE_HOME>"
		set "npm_config_userconfig=%NODE_HOME%/node_modules/npm/.npmrc"
		set "PATH=%NODE_HOME%;%NODE_HOME%/bin;"
	"#}.replace("<NODE_HOME>",&crate::simple_tray::resource_dir(node_dir).to_string_lossy().replace("\\","/"));
	#[cfg(not(windows))]
	let cmd_pre = indoc! {r#"
		#!/bin/bash
		NODE_HOME=<NODE_HOME>
		npm_config_userconfig=$NODE_HOME/node_modules/npm/.npmrc
		PATH=$NODE_HOME:$NODE_HOME/bin"
	"#}.replace("<NODE_HOME>",&crate::simple_tray::resource_dir(node_dir).to_string_lossy().replace("\\","/"));
	let cmd = format!("{}\n{}",cmd_pre,cmd);
	cmd
}