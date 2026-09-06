
use std::sync::OnceLock;

/// 全局 AppHandle（setup 时缓存，全局可用）
static APP: OnceLock<tauri::AppHandle> = OnceLock::new();
/// 主线程ID
static MAIN_THREAD_ID: OnceLock<std::thread::ThreadId> = OnceLock::new();

/// 获取全局 AppHandle（未初始化时返回 None）
pub fn app() -> Option<&'static tauri::AppHandle> {
    APP.get()
}

/// 初始化全局 AppHandle
pub(super) fn app_set(handler: tauri::AppHandle) {
	let _ = APP.set(handler);
}

/// 判断当前是否主线程（setup 在 run 开始时记录）
pub fn is_main_thread() -> bool {
    match MAIN_THREAD_ID.get() {
        Some(main) => std::thread::current().id() == *main,
        None => true, // 未记录时保守假定在主线程
    }
}

/// 初始化主线程ID
pub(super) fn main_thread_set(id: std::thread::ThreadId) {
	let _ = MAIN_THREAD_ID.set(id);
}