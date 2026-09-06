
use std::sync::OnceLock;

static HOOK_BEFORE_TRAY: OnceLock<Option<fn() -> Result<(), String>>> = OnceLock::new();
static HOOK_AFTER_TRAY:  OnceLock<Option<fn() -> Result<(), String>>> = OnceLock::new();
static HOOK_QUIT:        OnceLock<Option<fn() -> Result<(), String>>> = OnceLock::new();

/// 注册托盘生命周期钩子（编译期宏 hooks! 生成后调用此函数）
pub fn set_hooks(
    on_tray_before: Option<fn() -> Result<(), String>>,
    on_tray_after:  Option<fn() -> Result<(), String>>,
    on_quit:        Option<fn() -> Result<(), String>>,
) {
    let _ = HOOK_BEFORE_TRAY.set(on_tray_before);
    let _ = HOOK_AFTER_TRAY.set(on_tray_after);
    let _ = HOOK_QUIT.set(on_quit);
}

pub(crate) fn trigger_tray_before() -> Result<(), String> {
	if let Some(f) = HOOK_BEFORE_TRAY.get().and_then(|h| *h) {
        return f();
    }
    Ok(())
}

pub(crate) fn trigger_tray_after() {
	if let Some(f) = HOOK_AFTER_TRAY.get().and_then(|h| *h) {
        let _ = f();
    }
}

pub(crate) fn trigger_quit() {
	if let Some(f) = HOOK_QUIT.get().and_then(|h| *h) {
        let _ = f();
    }
}