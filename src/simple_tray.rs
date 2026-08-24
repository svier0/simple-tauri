/**
 * **注意**
 * 不论何种原因，此文件除了用户，禁止修改，禁止任何编辑
 */

pub use simple_tauri_macros::set_window_list;
pub use simple_tauri_macros::set_ipc_cmds;
pub use simple_tauri_macros::set_tray_menu;
pub use simple_tauri_macros::hooks;
pub use simple_tauri_macros::mutex;
pub use simple_tauri_macros::run;

use std::sync::{OnceLock};
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::menu::{CheckMenuItem, MenuItem, MenuBuilder, MenuItemBuilder, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Manager, WebviewWindowBuilder};

static QUIT_FLAG: AtomicBool = AtomicBool::new(false);
static LIGHT_MODE: AtomicBool = AtomicBool::new(false);
static LIGHT_CLOSE: AtomicBool = AtomicBool::new(false);
static LIGHT_ITEM: OnceLock<CheckMenuItem<tauri::Wry>> = OnceLock::new();
static TOGGLE_ITEM: OnceLock<MenuItem<tauri::Wry>> = OnceLock::new();
static EXTRA_ITEMS: OnceLock<Vec<(&'static str, &'static str, Option<fn()>)>> = OnceLock::new();
static IPC_HANDLER: OnceLock<Box<dyn Fn(tauri::ipc::Invoke) -> bool + Send + Sync>> = OnceLock::new();
static HOOK_BEFORE_TRAY: OnceLock<Option<fn() -> Result<(), String>>> = OnceLock::new();
static HOOK_AFTER_TRAY: OnceLock<Option<fn() -> Result<(), String>>> = OnceLock::new();
static HOOK_QUIT: OnceLock<Option<fn() -> Result<(), String>>> = OnceLock::new();
static MAIN_THREAD_ID: OnceLock<std::thread::ThreadId> = OnceLock::new();
/// 全局 AppHandle（setup 时缓存，全局可用）
static APP: OnceLock<tauri::AppHandle> = OnceLock::new();

/// 获取全局 AppHandle（未初始化时返回 None）
pub fn app() -> Option<&'static tauri::AppHandle> {
    APP.get()
}

/// 返回资源目录绝对路径（windows：exe 所在目录，非 windows：app 资源目录）
/// sub 非空时拼接子路径返回
pub fn resource_dir(sub: &str) -> std::path::PathBuf {
    use std::sync::OnceLock;
    static BASE: OnceLock<std::path::PathBuf> = OnceLock::new();
    let base = BASE.get_or_init(|| {
        #[cfg(windows)]
        {
            std::env::current_exe()
                .ok()
                .and_then(|exe| exe.parent().map(|d| d.to_path_buf()))
                .unwrap_or_else(|| std::path::PathBuf::from("."))
        }
        #[cfg(not(windows))]
        {
            app()
                .and_then(|a| a.path().resource_dir().ok())
                .unwrap_or_else(|| std::path::PathBuf::from("."))
        }
    });
    if sub.is_empty() {
        base.clone()
    } else {
        base.join(sub)
    }
}

/// 判断当前是否主线程（setup 在 run 开始时记录）
fn is_main_thread() -> bool {
    match MAIN_THREAD_ID.get() {
        Some(main) => std::thread::current().id() == *main,
        None => true, // 未记录时保守假定在主线程
    }
}

#[cfg(windows)]
pub fn run(context: tauri::Context<tauri::Wry>) {
    let _ = EXTRA_ITEMS.get_or_init(|| vec![]);

    tauri::Builder::default()
        .invoke_handler(IPC_HANDLER.get_or_init(|| Box::new(|_| false)))
        .on_window_event(|window, event| {
            // 关闭窗口时
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // 轻量模式下直接销毁窗口
                if LIGHT_CLOSE.swap(false, Ordering::SeqCst) {
                    return;
                }
                // 用户点关闭按钮或 close_window()：隐藏窗口，不退出
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .setup(move |app| {
            let _ = MAIN_THREAD_ID.set(std::thread::current().id());
            let _ = APP.set(app.handle().clone());
            let handle = app.handle().clone();

            // hook（含 wait_port 阻塞）放后台线程，成功后再调度回主线程建托盘
            // setup 立即返回，窗口/事件循环不阻塞；托盘等服务器起来才出
            std::thread::spawn(move || {
                if let Some(f) = HOOK_BEFORE_TRAY.get().and_then(|h| *h) {
                    if let Err(e) = f() {
                        fatal(&handle, &e);
                    }
                }
                // 回主线程建托盘（Tauri UI 必须在主线程），建好后执行 on_tray_after
                let h2 = handle.clone();
                let _ = handle.run_on_main_thread(move || {
                    create_tray(&h2);
                    if let Some(g) = HOOK_AFTER_TRAY.get().and_then(|h| *h) {
                        let _ = g();
                    }
                });
            });

            Ok(())
        })
        .build(context)
        .expect("error while building tauri application")
        .run(|_app_handle, event| {
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                if !QUIT_FLAG.load(Ordering::SeqCst) {
                    api.prevent_exit();
                }
            }
        });
}

/// 依据 is_running() 真实状态刷新菜单按钮文本。
fn refresh_toggle_text() {
    let running = crate::simple_serve::is_running();
    let text = if running { "停止" } else { "启动" }.to_string();
    tauri::async_runtime::spawn(async move {
        if let Some(item) = TOGGLE_ITEM.get() {
            let _ = item.set_text(text);
        }
    });
}

/// 创建托盘（必须在主线程调用，由 hook 成功后调度回主线程执行）
fn create_tray(app: &tauri::AppHandle) {
    // 声明托盘菜单项
    let toggle = MenuItemBuilder::with_id("toggle", "启动").build(app).unwrap();
    let _ = TOGGLE_ITEM.set(toggle.clone());
    let light = CheckMenuItem::with_id(app, "light", "轻量模式", true, false, None::<&str>).unwrap();
    let _ = LIGHT_ITEM.set(light.clone());
    let quit = MenuItemBuilder::with_id("quit", "退出").build(app).unwrap();

    // 静默启动：配置开启则只留托盘不建窗口
    let silent = true;
    LIGHT_MODE.store(silent, Ordering::SeqCst);
    if let Some(item) = LIGHT_ITEM.get() {
        let _ = item.set_checked(silent);
    }
    if !silent {
        show_window("main");
    }

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

    // 创建托盘
    let tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .show_menu_on_left_click(false)
        .show_menu_on_right_click(false)
        .on_tray_icon_event(|_tray, event| {
            // 左键弹起
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_window("main");
            }
            // 右键弹起
            if let TrayIconEvent::Click {
                button: MouseButton::Right,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                refresh_toggle_text();
                if let Some(menu) = _tray.get_menu() {
                    _tray.popup_menu(&menu);
                }
            }
        })
        .menu(&menu)
        .on_menu_event(move |app, event| match event.id.as_ref() {
            "show" => show_window("main"),
            "toggle" => {
                if crate::simple_serve::is_running() {
                    crate::simple_server::stop();
                } else {
                    crate::simple_server::restart();
                }
            }
            "light" => {
                let is_light = !LIGHT_MODE.load(Ordering::SeqCst);
                LIGHT_MODE.store(is_light, Ordering::SeqCst);
                if let Some(item) = LIGHT_ITEM.get() {
                    let _ = item.set_checked(is_light);
                }
                if is_light {
                    if let Some(w) = app.get_webview_window("main") {
                        LIGHT_CLOSE.store(true, Ordering::SeqCst);
                        let _ = w.close();
                    }
                } else {
                    show_window("main");
                }
            }
            "quit" => {
                QUIT_FLAG.store(true, Ordering::SeqCst);
                if let Some(f) = HOOK_QUIT.get().and_then(|h| *h) {
                    let _ = f();
                }
                app.exit(0);
            }
            id => {
                if let Some((_, _, Some(cb))) = EXTRA_ITEMS
                    .get().unwrap().iter()
                    .find(|(i, _, _)| *i == id)
                {
                    cb();
                }
            }
        })
        .build(app).unwrap();

    // 托盘必须保持存活，否则应用会在托盘图标创建后立即退出
    app.manage(TrayState { _tray: tray });
}

/// 设置ipc命令
pub fn set_ipc_cmds(
    commands: impl Fn(tauri::ipc::Invoke) -> bool + Send + Sync + 'static,
) {
    let _ = IPC_HANDLER.set(Box::new(commands));
}

/// 窗口配置
#[derive(Clone, Copy)]
pub struct WindowConfig {
    pub id: &'static str,
    pub title: &'static str,
    pub url: &'static str,
    pub width: f64,
    pub height: f64,
    pub decorations: bool,
}

static WND_LIST: OnceLock<&'static [WindowConfig]> = OnceLock::new();

/// 设置窗口列表（编译期宏 set_window_list! 生成静态数组后调用此函数）
pub fn set_window_list(wnd_list: &'static [WindowConfig]) {
    let _ = WND_LIST.set(wnd_list);
}

/// 注册托盘生命周期钩子（编译期宏 hooks! 生成后调用此函数）
pub fn set_hooks(
    on_tray_before: Option<fn() -> Result<(), String>>,
    on_tray_after: Option<fn() -> Result<(), String>>,
    on_quit: Option<fn() -> Result<(), String>>,
) {
    let _ = HOOK_BEFORE_TRAY.set(on_tray_before);
    let _ = HOOK_AFTER_TRAY.set(on_tray_after);
    let _ = HOOK_QUIT.set(on_quit);
}

/// 设置托盘菜单（编译期宏 set_tray_menu! 生成静态数组后调用此函数）
pub fn set_tray_menu(menu: &'static [(&'static str, &'static str, Option<fn()>)]) {
    let _ = EXTRA_ITEMS.set(menu.iter().copied().collect());
}

/// 创建主窗口：先隐藏，等 webview 页面加载完成后（on_page_load）再显示，避免白屏闪烁。
fn build_window(app: &tauri::AppHandle, wndid: &str) -> tauri::Result<()> {
    let wnd = WND_LIST
        .get()
        .and_then(|list| list.iter().find(|w| w.id == wndid))
        .ok_or_else(|| {
            tauri::Error::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("window {wndid} 未在窗口列表中声明"),
            ))
        })?;

    let url = if wnd.url.starts_with("http://") || wnd.url.starts_with("https://") {
        tauri::WebviewUrl::External(wnd.url.parse().map_err(tauri::Error::InvalidUrl)?)
    } else {
        tauri::WebviewUrl::App(wnd.url.into())
    };

    WebviewWindowBuilder::new(app, wndid, url)
        .title(wnd.title)
        .inner_size(wnd.width, wnd.height)
        .center()
        .visible(false)
        .background_color(tauri::window::Color(0x22, 0x22, 0x22, 0xFF))
        .on_page_load(|webview, payload| {
            if payload.event() == tauri::webview::PageLoadEvent::Finished {
                let _ = webview.show();
                let _ = webview.set_focus();
            }
        })
        .decorations(wnd.decorations)
        .build()?;
    Ok(())
}

/// 显示主窗口：主窗口未创建或已销毁则创建
/// 可在任意线程调用：非主线程时自动调度回主线程执行（窗口 API 必须在主线程）
pub fn show_window(wndid: &str) {
    let Some(app) = app() else { return };
    if !is_main_thread() {
        let app_receiver = app.clone();
        let wndid = wndid.to_string();
        let _ = app_receiver.run_on_main_thread(move || show_window(&wndid));
        return;
    }
    if LIGHT_MODE.swap(false, Ordering::SeqCst) {
        if let Some(item) = LIGHT_ITEM.get() {
            let _ = item.set_checked(false);
        }
    }
    if let Some(w) = app.get_webview_window(wndid) {
        let _ = w.show();
        let _ = w.set_focus();
    } else {
        let _ = build_window(app, wndid);
    }
}

/// 关闭（销毁）窗口：存在则关闭，不存在则忽略
/// 可在任意线程调用：非主线程时自动调度回主线程执行（窗口 API 必须在主线程）
pub fn close_window(wndid: &str) {
    let Some(app) = app() else { return };
    if !is_main_thread() {
        let app_receiver = app.clone();
        let wndid = wndid.to_string();
        let _ = app_receiver.run_on_main_thread(move || close_window(&wndid));
        return;
    }
    if let Some(w) = app.get_webview_window(wndid) {
        let _ = w.close();
    }
}

/// 在指定窗口的 webview 里执行一段 JavaScript
/// 可在任意线程调用：非主线程时自动调度回主线程执行（eval 必须在主线程）
pub fn runjs(wndid: &str, script: &str) {
    let Some(app) = app() else { return };
    if !is_main_thread() {
        let app_receiver = app.clone();
        let wndid = wndid.to_string();
        let script = script.to_string();
        let _ = app_receiver.run_on_main_thread(move || runjs(&wndid, &script));
        return;
    }
    if let Some(w) = app.get_webview_window(wndid) {
        let _ = w.eval(script);
    }
}

struct TrayState {
    _tray: tauri::tray::TrayIcon<tauri::Wry>,
}

/// 弹致命错误框并退出进程
#[cfg(windows)]
fn fatal(app: &tauri::AppHandle, msg: &str) -> ! {
    use std::os::windows::ffi::OsStrExt;
    let wide: Vec<u16> = std::ffi::OsStr::new(msg)
        .encode_wide()
        .chain(Some(0))
        .collect();
    let title: String = app.config().product_name.clone().unwrap_or_default();
    let title_wide: Vec<u16> = std::ffi::OsStr::new(&title)
        .encode_wide()
        .chain(Some(0))
        .collect();
    unsafe {
        windows_sys::Win32::UI::WindowsAndMessaging::MessageBoxW(
            std::ptr::null_mut(),
            wide.as_ptr(),
            title_wide.as_ptr(),
            0x10, // MB_ICONERROR
        )
    };
    app.exit(1);
    std::process::exit(1);
}

#[cfg(not(windows))]
fn fatal(app: &tauri::AppHandle, msg: &str) -> ! {
    eprintln!("fatal: {msg}");
    app.exit(1);
    std::process::exit(1);
}