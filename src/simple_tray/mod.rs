/**
 * **注意**
 * 不论何种原因，此文件除了用户，禁止修改，禁止任何编辑
 */

pub use simple_tauri_macros::set_window_list;
pub use simple_tauri_macros::set_ipc_cmds;
pub use simple_tauri_macros::set_tray_menu;
pub use simple_tauri_macros::hooks;
pub use simple_tauri_macros::run;
pub use simple_tauri_macros::ipc_result;

mod app_handler;
mod window_list;
mod tray_menu;
mod tray_hook;
pub use app_handler::*;
pub use window_list::*;
pub use tray_menu::*;
pub use tray_hook::*;

use std::sync::{OnceLock};
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Manager, WebviewWindowBuilder};

static QUIT_FLAG: AtomicBool = AtomicBool::new(false);
static IPC_HANDLER: OnceLock<Box<dyn Fn(tauri::ipc::Invoke) -> bool + Send + Sync>> = OnceLock::new();
static PRODUCT_NAME: OnceLock<String> = OnceLock::new();

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
            app().path().resource_dir().ok()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
        }
    });
    if sub.is_empty() {
        base.clone()
    } else {
        base.join(sub.replace("/",std::path::MAIN_SEPARATOR_STR))
    }
}

pub fn product_name() -> String {
    PRODUCT_NAME.get_or_init(|| String::from("")).clone()
}

#[cfg(windows)]
pub fn run(context: tauri::Context<tauri::Wry>) {
    let _ = PRODUCT_NAME.set(context.config().product_name.clone().unwrap_or_default());
    tauri::Builder::default()
        .invoke_handler(IPC_HANDLER.get_or_init(|| Box::new(|_| false)))
        .on_window_event(|window, event| {
            // 关闭窗口时
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // 轻量模式下直接销毁窗口
                if tray_menu::light_close_off() {
                    return;
                }
                // 用户点关闭按钮或 close_window()：隐藏窗口，不退出
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .setup(move |app_handle| {
            main_thread_set(std::thread::current().id());
            app_set(app_handle.handle().clone());

            // hook（含 wait_port 阻塞）放后台线程，成功后再调度回主线程建托盘
            // setup 立即返回，窗口/事件循环不阻塞；托盘等服务器起来才出
            std::thread::spawn(move || {
                if let Err(e) = trigger_tray_before() {
                    fatal(&e);
                }
                // 回主线程建托盘（Tauri UI 必须在主线程），建好后执行 on_tray_after
                let _ = app().run_on_main_thread(move || {
                    create_tray();
                    trigger_tray_after();
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

/// 创建托盘（必须在主线程调用，由 hook 成功后调度回主线程执行）
fn create_tray() {
    let app = app();

    // 创建托盘菜单
    let menu = tray_menu::build_tray_menu();

    // 创建托盘
    let tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .show_menu_on_left_click(false)
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
            // 右键按下
            if let TrayIconEvent::Click {
                button: MouseButton::Right,
                button_state: MouseButtonState::Down,
                ..
            } = event
            {
                // 弹出前刷新菜单
                tray_menu::refresh_menu();
            }
        })
        .menu(&menu)
        .on_menu_event(move |_app, event| tray_menu::trigger_tray_menu_cb(event.id.as_ref()) )
        .build(app).unwrap();

    // 托盘必须保持存活，否则应用会在托盘图标创建后立即退出
    app.manage(TrayState { _tray: tray });
}

/// 退出
pub fn app_quit(){
    QUIT_FLAG.store(true, Ordering::SeqCst);
    trigger_quit();
    app().exit(0);
}

/// 设置ipc命令
pub fn set_ipc_cmds(
    commands: impl Fn(tauri::ipc::Invoke) -> bool + Send + Sync + 'static,
) {
    let _ = IPC_HANDLER.set(Box::new(commands));
}

/// 创建主窗口：先隐藏，等 webview 页面加载完成后（on_page_load）再显示，避免白屏闪烁。
fn build_window(wndid: &str) -> tauri::Result<()> {
    let app = app();
    let wnd = window_list_find(wndid)?;

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
    let app = app();
    if !is_main_thread() {
        let app_receiver = app.clone();
        let wndid = wndid.to_string();
        let _ = app_receiver.run_on_main_thread(move || show_window(&wndid));
        return;
    }
    // 关闭轻量模式
    tray_menu::toggle_light_mode(2);

    if let Some(w) = app.get_webview_window(wndid) {
        let _ = w.show();
        let _ = w.set_focus();
    } else {
        let _ = build_window(wndid);
    }
}

/// 关闭（销毁）窗口：存在则关闭，不存在则忽略
/// 可在任意线程调用：非主线程时自动调度回主线程执行（窗口 API 必须在主线程）
pub fn close_window(wndid: &str) {
    let app = app();
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
    let app = app();
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
fn fatal(msg: &str) -> ! {
    let app = app();
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
fn fatal(msg: &str) -> ! {
    let app = app();
    eprintln!("fatal: {msg}");
    app.exit(1);
    std::process::exit(1);
}