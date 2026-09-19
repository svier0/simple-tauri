
use std::sync::OnceLock;

static WND_LIST: OnceLock<Vec<WindowConfig>> = OnceLock::new();

/// 窗口配置
#[derive(Clone)]
pub struct WindowConfig {
    pub id: String,
    pub title: String,
    pub url: String,
    pub width: f64,
    pub height: f64,
    pub decorations: bool,
}

/// 设置窗口列表
pub fn set_window_list(wnd_list: Vec<WindowConfig>) {
    let _ = WND_LIST.set(wnd_list);
}

pub(super) fn window_list_find(wndid: &str) -> tauri::Result<&'static WindowConfig> {
    WND_LIST
        .get()
        .and_then(|list| list.iter().find(|w| w.id == wndid))
        .ok_or_else(|| {
            tauri::Error::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("window {wndid} 未在窗口列表中声明"),
            ))
        })
}