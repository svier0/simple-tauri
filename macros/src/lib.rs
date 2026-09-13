use proc_macro::TokenStream;

mod simple_tray;
mod simple_serve;
mod mod_;
mod utils;
mod ipc_result;
mod config;

/// 单实例互斥体检查（Windows），已有实例运行时 `return` 退出
/// 不传参时自动用当前 crate 包名（CARGO_PKG_NAME），也可显式传字符串字面量，如 mutex!("myapp")
/// 用法: mutex!() 或 mutex!("myapp")，应放在启动代码最前
#[proc_macro]
pub fn mutex(input: TokenStream) -> TokenStream {
    mod_::mutex_impl(input)
}

/// 启动应用：读取 simple_tauri.jsonc 配置并展开所有初始化代码
/// 用法: simple_tray::run!();
#[proc_macro]
pub fn run(input: TokenStream) -> TokenStream {
    simple_tray::run_impl(input)
}

/// 简化 set_start_cmd 调用：set_start_cmd!("run.bat") 或 set_start_cmd!("run_{}.bat", ver)
#[proc_macro]
pub fn set_start_cmd(input: TokenStream) -> TokenStream {
    simple_serve::set_start_cmd_impl(input)
}

/// 简化 sh2rs 调用：sh2rs!("wget url") 或 sh2rs!("wget -O {} {}", fname, url)
#[proc_macro]
pub fn sh2rs(input: TokenStream) -> TokenStream {
    utils::sh2rs_impl(input)
}

/// 简化 try_quote 调用：try_quote!("wget url") 或 try_quote!("wget -O {} {}", fname, url)
#[proc_macro]
pub fn try_quote(input: TokenStream) -> TokenStream {
    utils::try_quote_impl(input)
}

/// IPC 统一返回结构
/// ipc_result!(0, "", data)       → 成功
/// ipc_result!(1, "错误", null)   → 失败
/// ipc_result!(result)            → Result<T,String> 自动匹配
#[proc_macro]
pub fn ipc_result(input: TokenStream) -> TokenStream {
    ipc_result::ipc_result_impl(input)
}

/// 获取配置值: get!("key") → serde_json::Value
#[proc_macro]
pub fn get(input: TokenStream) -> TokenStream {
    config::get_impl(input)
}

/// 获取类型化配置值: get_or!("age", -1) → i64
/// 根据默认值类型自动选择 as_i64/as_u64/as_f64/as_bool/as_str
#[proc_macro]
pub fn get_or(input: TokenStream) -> TokenStream {
    config::get_or_impl(input)
}
