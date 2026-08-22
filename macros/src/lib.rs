use proc_macro::TokenStream;

mod simple_tray;
mod simple_serve;
mod mod_;
mod utils;

/// 编译期解析 JSON 窗口列表，生成静态 WindowConfig 数组
///
/// 格式: set_window_list!(r#"[ [wndid, title?, wburl?, wnd_width?, wnd_height?, wnd_decorations?], ... ]"#)
/// - title 为空时使用 ${wndid}
/// - wburl 为空时使用 ${wndid}.html
/// - wnd_width 默认 800.0
/// - wnd_height 默认 540.0
/// - wnd_decorations 默认 true
#[proc_macro]
pub fn set_window_list(input: TokenStream) -> TokenStream {
    simple_tray::set_window_list_impl(input)
}

/// 把前端 IPC 命令列表转发给 tauri::generate_handler!，注册到托盘运行器
/// 用法:
/// - set_ipc_cmds!() 无参：编译期自动扫描调用方 crate 的 src/lib.rs 顶层，
///   收集所有 #[tauri::command] 函数并注册（没有则注册空 handler）
/// - set_ipc_cmds!("ipc") / set_ipc_cmds!(["lib", "ipc"]) / set_ipc_cmds!(lib, ipc)：
///   扫描指定一个或多个模块，收集所有 #[tauri::command] 函数并注册
/// - set_ipc_cmds![a, b] 手动列出函数路径
#[proc_macro]
pub fn set_ipc_cmds(input: TokenStream) -> TokenStream {
    simple_tray::set_ipc_cmds_impl(input)
}

/// 编译期解析 JSON 托盘菜单，生成静态菜单数组
///
/// 格式: set_tray_menu!(r#"[ [id, label, cb], ... ]"#)
/// - id 为空 => 分隔线（None 回调）
/// - cb 为空 => None，内置 id(show/light/quit) 由托盘逻辑接管
/// - cb 为函数路径字符串，编译期解析为 fn()（无参，内部用全局 AppHandle）
#[proc_macro]
pub fn set_tray_menu(input: TokenStream) -> TokenStream {
    simple_tray::set_tray_menu_impl(input)
}

/// 注册托盘生命周期钩子
///
/// 格式: hooks!(on_tray_before, on_tray_after, on_quit)
/// 三个位置对应三个钩子，缺省用 `_` 占位（不注册），如 hooks!(on_tray_before, _, on_quit)
#[proc_macro]
pub fn hooks(input: TokenStream) -> TokenStream {
    simple_tray::hooks_impl(input)
}

/// 单实例互斥体检查（Windows），已有实例运行时 `return` 退出
/// 不传参时自动用当前 crate 包名（CARGO_PKG_NAME），也可显式传字符串字面量，如 mutex!("dsh")
/// 用法: mutex!() 或 mutex!("dsh")，应放在启动代码最前
#[proc_macro]
pub fn mutex(input: TokenStream) -> TokenStream {
    mod_::mutex_impl(input)
}

/// 启动应用：等价 simple_tray::run(tauri::generate_context!())
/// 由宏在调用方 crate 展开，读取该 crate 的 tauri.conf.json
#[proc_macro]
pub fn run(input: TokenStream) -> TokenStream {
    simple_tray::run_impl(input)
}

/// 简化 start 调用：start!("run.bat") 或 start!("run_{}.bat", ver)
#[proc_macro]
pub fn start(input: TokenStream) -> TokenStream {
    simple_serve::start_impl(input)
}

/// 简化 sh2rs 调用：sh2rs!("wget url") 或 sh2rs!("wget -O {} {}", fname, url)
#[proc_macro]
pub fn sh2rs(input: TokenStream) -> TokenStream {
    utils::sh2rs_impl(input)
}
