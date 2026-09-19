use crate::utils::sh2rs::sh2rs;
use crate::utils::sh2rs::get_stdout_buffer;

use std::sync::OnceLock;

static DEPS: OnceLock<Vec<String>> = OnceLock::new();

pub fn set_depsenv(deps: &[String]) {
    let _ = DEPS.set(deps.to_vec());
}

pub(super) fn ensure_depsenv() -> Result<(),String>{
    let deps: &[String] = DEPS.get().map_or(&[], |v| v.as_slice());
    for dep in deps {
        if dep == "node" || dep.starts_with("node@") {
            let ver = dep.strip_prefix("node@").unwrap_or("");
            #[cfg(windows)]
            sh2rs!("sh where node").ok();
            #[cfg(not(windows))]
            sh2rs!("sh which node").ok();
            let mut node_dir = get_stdout_buffer();
            if !node_dir.is_empty() {
                node_dir = std::path::Path::new(&node_dir).parent()
                    .map(|p| p.to_string_lossy().replace("\\","/"))
                    .unwrap_or_default();
            }
            crate::utils::ensure_node(ver, &node_dir)?;
        } else if dep == "pnpm" || dep.starts_with("pnpm@") {
            let ver = dep.strip_prefix("pnpm@").unwrap_or("");
            crate::utils::ensure_pnpm(ver)?;
        }
    }
    Ok(())
}