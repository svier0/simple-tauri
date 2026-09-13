
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::menu::{CheckMenuItem, MenuItem, MenuBuilder, MenuItemBuilder, PredefinedMenuItem};
use tauri::Manager;

// 轻量模式
static LIGHT_MODE: AtomicBool = AtomicBool::new(false);
static LIGHT_CLOSE: AtomicBool = AtomicBool::new(false);
// 轻量模式菜单
static LIGHT_ITEM: OnceLock<CheckMenuItem<tauri::Wry>> = OnceLock::new();
// 服务状态菜单
static TOGGLE_ITEM: OnceLock<MenuItem<tauri::Wry>> = OnceLock::new();
// 用户自定义菜单列表
static EXTRA_ITEMS: OnceLock<Vec<(&'static str, &'static str, Option<fn()>)>> = OnceLock::new();

// 托盘菜单回调-显示主窗口
fn cb_show(){
	super::show_window("main");
}

// 托盘菜单回调-切换服务状态
fn cb_toggle(){
    if crate::simple_serve::is_running() {
        let _ = crate::simple_serve::stop();
    } else {
        let _ = crate::simple_serve::start();
    }
}

// 托盘菜单回调-切换轻量模式
fn cb_light(){
    toggle_light_mode(0);
}

// 托盘菜单回调-退出
fn cb_quit(){
    super::app_quit();
}

// 获取托盘菜单回调 根据id获取
pub(super) fn trigger_tray_menu_cb(id: &str) {
    if let Some((_, _, Some(cb))) = EXTRA_ITEMS
        .get().unwrap().iter()
        .find(|(i, _, _)| *i == id)
    {
        cb();
    }
}

/// 设置托盘菜单（编译期宏 set_tray_menu! 生成静态数组后调用此函数）
pub fn set_tray_menu(menulist: &'static [(&'static str, &'static str, Option<fn()>)]) {
    let mut list: Vec<(&'static str, &'static str, Option<fn()>)> = Vec::new();
    for &(id, label, _) in menulist {
        match id {
            "show" => list.push((id, label, Some(cb_show))),
            "toggle" => list.push((id, "启动", Some(cb_toggle))),
            "light" => list.push((id, "轻量模式", Some(cb_light))),
            "quit" => {},
            _ => list.push((id, label, None)),
        }
    }
    list.push(("quit", "退出", Some(cb_quit)));
    let _ = EXTRA_ITEMS.set(list);
}

// 生成并返回菜单
pub(super) fn build_tray_menu() -> tauri::menu::Menu<tauri::Wry> {
    let _ = EXTRA_ITEMS.get_or_init(|| vec![]);

    let app = super::app();
    // 声明托盘菜单项
    let toggle = MenuItemBuilder::with_id("toggle", "启动").build(app).unwrap();
    let _ = TOGGLE_ITEM.set(toggle.clone());
    let light = CheckMenuItem::with_id(app, "light", "轻量模式", true, false, None::<&str>).unwrap();
    let _ = LIGHT_ITEM.set(light.clone());
    let quit = MenuItemBuilder::with_id("quit", "退出").build(app).unwrap();

    // 创建托盘菜单
    let mut menu = MenuBuilder::new(app);
    for &(id, label, _) in EXTRA_ITEMS.get().unwrap() {
        if id=="" {
            menu = menu.item(&PredefinedMenuItem::separator(app).unwrap());
        }else if id=="toggle" {
            menu = menu.item(&toggle);
        }else if id=="light" {
            menu = menu.item(&light);
        }else{
            menu = menu.item(&MenuItemBuilder::with_id(id, label).build(app).unwrap());
        }
    }
    let menu = menu.item(&PredefinedMenuItem::separator(app).unwrap())
        .item(&quit)
        .build().unwrap();

    // 静默启动：默认开启轻量模式
    if crate::config::get_or!("silent_launch",false) {
        toggle_light_mode(1);
    }

    menu
}

// 是否轻量模式下的关闭 并设置为false (给窗口关闭事件里用的)
pub(super) fn light_close_off() -> bool {
    LIGHT_CLOSE.swap(false, Ordering::SeqCst)
}

/// 切换轻量模式
/// 0: 切换 1：开启 2：关闭
pub fn toggle_light_mode(flag: i32) {
    let is_light = match flag {
        1 => true,
        2 => false,
        _ => !LIGHT_MODE.load(Ordering::SeqCst),
    };
    LIGHT_MODE.store(is_light, Ordering::SeqCst);
    if let Some(item) = LIGHT_ITEM.get() {
        let _ = item.set_checked(is_light);
    }
    if is_light {
        let app = super::app();
        if let Some(w) = app.get_webview_window("main") {
            LIGHT_CLOSE.store(true, Ordering::SeqCst);
            let _ = w.close();
        }
    }
}

/// 依据 is_running() 真实状态刷新菜单按钮文本。
fn refresh_toggle_text() {
    let running = crate::simple_serve::is_running();
    let text = if running { "停止" } else { "启动" }.to_string();
    if let Some(item) = TOGGLE_ITEM.get() {
        let _ = item.set_text(text);
    }
}

/// 刷新菜单
pub(super) fn refresh_menu() {
    refresh_toggle_text();
}