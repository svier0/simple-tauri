
use std::sync::Mutex;

static WND_LIST: Mutex<Vec<WindowConfig>> = Mutex::new(Vec::new());

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
    *WND_LIST.lock().unwrap() = wnd_list;
}

pub(super) fn window_list_find(wndid: &str) -> tauri::Result<WindowConfig> {
    WND_LIST
        .lock()
        .unwrap()
        .iter()
        .find(|w| w.id == wndid)
        .cloned()
        .ok_or_else(|| not_found(wndid))
}

/// 修改列表指定窗口数据：通过闭包修改 id 对应项
pub fn edit_window_item<F: FnOnce(&mut WindowConfig)>(id: &str, f: F) -> Result<(), String> {
    let mut list = WND_LIST.lock().unwrap();
    let item = list
        .iter_mut()
        .find(|w| w.id == id)
        .ok_or_else(|| format!("window {id} 未在窗口列表中声明"))?;
    f(item);
    Ok(())
}

fn not_found(wndid: &str) -> tauri::Error {
    tauri::Error::Io(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        format!("window {wndid} 未在窗口列表中声明"),
    ))
}
