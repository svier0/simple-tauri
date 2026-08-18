use proc_macro::TokenStream;
use proc_macro2::Literal;
use quote::quote;
use syn::{parse_macro_input, LitStr};

/// 容忍 JSON 字符串里的尾随逗号（`[,]` / `[,}`），serde_json 严格模式不允许
/// 注意：必须按 char 处理，不能按字节，否则会破坏多字节 UTF-8 中文
fn strip_trailing_commas(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == ',' {
            let mut j = i + 1;
            while j < chars.len() && (chars[j] == ' ' || chars[j] == '\t' || chars[j] == '\n' || chars[j] == '\r') {
                j += 1;
            }
            if j < chars.len() && (chars[j] == ']' || chars[j] == '}') {
                // 跳过逗号，保留后续的 ] 或 }
                i = j;
                continue;
            }
        }
        out.push(c);
        i += 1;
    }
    out
}

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
    let lit = parse_macro_input!(input as LitStr);
    let json: serde_json::Value = serde_json::from_str(&strip_trailing_commas(&lit.value()))
        .expect("set_window_list!: JSON 解析失败");

    let rows = json.as_array().expect("set_window_list!: 顶层必须是数组");
    let mut items = Vec::new();

    for row in rows {
        let row = row.as_array().expect("set_window_list!: 每个窗口必须是数组");
        let id = row
            .first()
            .and_then(|v| v.as_str())
            .expect("set_window_list!: 缺少窗口 id")
            .to_string();

        let title = row
            .get(1)
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| id.clone());

        let url = row
            .get(2)
            .and_then(|v| v.as_str())
            .map(str::to_string)
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| format!("{id}.html"));

        let width = row.get(3).and_then(|v| v.as_f64()).unwrap_or(800.0);
        let height = row.get(4).and_then(|v| v.as_f64()).unwrap_or(540.0);
        let decorations = row.get(5).and_then(|v| v.as_bool()).unwrap_or(true);

        let id = Literal::string(&id);
        let title = Literal::string(&title);
        let url = Literal::string(&url);

        items.push(quote! {
            ::simple_tauri::simple_tray::WindowConfig {
                id: #id,
                title: #title,
                url: #url,
                width: #width,
                height: #height,
                decorations: #decorations,
            }
        });
    }

    let expanded = quote! {
        ::simple_tauri::simple_tray::set_window_list(&[#(#items),*]);
    };
    expanded.into()
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
    let input = proc_macro2::TokenStream::from(input);

    // 空输入 => 默认扫 lib
    let modules: Vec<String> = if input.is_empty() {
        vec!["lib".to_string()]
    } else {
        // 按逗号拆成若干项
        let mut groups: Vec<Vec<proc_macro2::TokenTree>> = Vec::new();
        let mut cur: Vec<proc_macro2::TokenTree> = Vec::new();
        for tt in input.clone() {
            if matches!(&tt, proc_macro2::TokenTree::Punct(p) if p.as_char() == ',') {
                groups.push(cur);
                cur = Vec::new();
            } else {
                cur.push(tt);
            }
        }
        if !cur.is_empty() {
            groups.push(cur);
        }

        // 每项若是单个字符串字面量或标识符 => 视为模块名；否则手动列表
        let mut mods = Vec::new();
        let mut manual = false;
        for g in &groups {
            match g.as_slice() {
                [proc_macro2::TokenTree::Literal(l)] => {
                    if let Ok(lit) = syn::parse2::<LitStr>(quote!(#l)) {
                        mods.push(lit.value());
                    } else {
                        manual = true;
                    }
                }
                [proc_macro2::TokenTree::Ident(i)] => mods.push(i.to_string()),
                _ => manual = true,
            }
        }
        if manual {
            let expanded = quote! {
                ::simple_tauri::simple_tray::set_ipc_cmds(tauri::generate_handler![#input]);
            };
            return expanded.into();
        }
        mods
    };

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("set_ipc_cmds!: CARGO_MANIFEST_DIR 未设置");

    let mut all_paths: Vec<proc_macro2::TokenStream> = Vec::new();
    for module in &modules {
        let ipc_path = std::path::Path::new(&manifest_dir)
            .join("src")
            .join(format!("{module}.rs"));
        let src = std::fs::read_to_string(&ipc_path).unwrap_or_else(|e| {
            panic!("set_ipc_cmds!: 读取 {ipc_path:?} 失败: {e}")
        });

        let file: syn::File = syn::parse_file(&src)
            .unwrap_or_else(|e| panic!("set_ipc_cmds!: {module}.rs 解析失败: {e}"));
        let mut names = Vec::new();
        for item in &file.items {
            if let syn::Item::Fn(f) = item {
                let is_cmd = f.attrs.iter().any(|a| {
                    a.path()
                        .segments
                        .last()
                        .map(|s| s.ident == "command")
                        .unwrap_or(false)
                });
                if is_cmd {
                    names.push(f.sig.ident.clone());
                }
            }
        }

        let module_ident =
            syn::Ident::new(&module.replace('-', "_"), proc_macro2::Span::call_site());
        for n in names {
            if module == "lib" {
                all_paths.push(quote!(#n));
            } else {
                let module = module_ident.clone();
                all_paths.push(quote!(#module::#n));
            }
        }
    }

    // 扫描不到任何 command 时注册空 handler，等价于没有 IPC 命令
    if all_paths.is_empty() {
        return quote! {
            ::simple_tauri::simple_tray::set_ipc_cmds(|_| false);
        }
        .into();
    }

    quote! {
        ::simple_tauri::simple_tray::set_ipc_cmds(tauri::generate_handler![#(#all_paths),*]);
    }
    .into()
}

/// 编译期解析 JSON 托盘菜单，生成静态菜单数组
///
/// 格式: set_tray_menu!(r#"[ [id, label, cb], ... ]"#)
/// - id 为空 => 分隔线（None 回调）
/// - cb 为空 => None，内置 id(show/light/quit) 由托盘逻辑接管
/// - cb 为函数路径字符串，编译期解析为 fn()（无参，内部用全局 AppHandle）
#[proc_macro]
pub fn set_tray_menu(input: TokenStream) -> TokenStream {
    let lit = parse_macro_input!(input as LitStr);
    let json: serde_json::Value = serde_json::from_str(&strip_trailing_commas(&lit.value()))
        .expect("set_tray_menu!: JSON 解析失败");

    let rows = json.as_array().expect("set_tray_menu!: 顶层必须是数组");
    let mut items = Vec::new();

    for row in rows {
        let row = row.as_array().expect("set_tray_menu!: 每个菜单项必须是数组");
        let id = row.first().and_then(|v| v.as_str()).unwrap_or_default();
        let label = row.get(1).and_then(|v| v.as_str()).unwrap_or_default();

        if id.is_empty() {
            items.push(quote! {
                ("", "", None)
            });
            continue;
        }

        let cb_str = row.get(2).and_then(|v| v.as_str()).unwrap_or_default();
        let cb: Option<syn::Path> = if cb_str.is_empty() {
            None
        } else {
            Some(
                syn::parse_str(cb_str)
                    .expect("set_tray_menu!: 回调必须是函数路径字符串, 如 \"show_setting\""),
            )
        };

        let id = Literal::string(id);
        let label = Literal::string(label);

        items.push(match cb {
            Some(cb) => quote! {
                (#id, #label, Some(#cb as fn()))
            },
            None => quote! {
                (#id, #label, None)
            },
        });
    }

    let expanded = quote! {
        ::simple_tauri::simple_tray::set_tray_menu(&[#(#items),*]);
    };
    expanded.into()
}

/// 注册托盘生命周期钩子
///
/// 格式: hooks!(on_tray_before, on_tray_after, on_quit)
/// 三个位置对应三个钩子，缺省用 `_` 占位（不注册），如 hooks!(on_tray_before, _, on_quit)
#[proc_macro]
pub fn hooks(input: TokenStream) -> TokenStream {
    let tokens: Vec<proc_macro2::TokenTree> =
        proc_macro2::TokenStream::from(input).into_iter().collect();
    let mut paths: Vec<Option<syn::Path>> = Vec::new();
    let mut current: Vec<proc_macro2::TokenTree> = Vec::new();
    for tt in tokens {
        if matches!(&tt, proc_macro2::TokenTree::Punct(p) if p.as_char() == ',') {
            paths.push(parse_hook_arg(current));
            current = Vec::new();
        } else {
            current.push(tt);
        }
    }
    if !current.is_empty() {
        paths.push(parse_hook_arg(current));
    }

    let hook = |i: usize| {
        paths
            .get(i)
            .and_then(|p| p.as_ref())
            .map(|p| {
                quote!(
                    Some(#p as fn() -> ::std::result::Result<(), ::std::string::String>)
                )
            })
            .unwrap_or(quote!(None))
    };
    let before = hook(0);
    let after = hook(1);
    let quit = hook(2);

    let expanded = quote! {
        ::simple_tauri::simple_tray::set_hooks(
            #before, #after, #quit,
        );
    };
    expanded.into()
}

fn parse_hook_arg(tts: Vec<proc_macro2::TokenTree>) -> Option<syn::Path> {
    let is_underscore = matches!(
        tts.as_slice(),
        [proc_macro2::TokenTree::Ident(i)] if i == "_"
    );
    if is_underscore {
        return None;
    }
    let stream: proc_macro2::TokenStream = tts.into_iter().collect();
    Some(
        syn::parse2::<syn::Path>(stream)
            .expect("hooks!: 需要函数名或下划线占位符 _"),
    )
}

/// 单实例互斥体检查（Windows），已有实例运行时 `return` 退出
/// 不传参时自动用当前 crate 包名（CARGO_PKG_NAME），也可显式传字符串字面量，如 mutex!("dsh")
/// 用法: mutex!() 或 mutex!("dsh")，应放在启动代码最前
#[proc_macro]
pub fn mutex(input: TokenStream) -> TokenStream {
    let name = if input.is_empty() {
        None
    } else {
        let lit = parse_macro_input!(input as LitStr);
        Some(Literal::string(&lit.value()))
    };

    match &name {
        Some(name) => quote! {
            #[cfg(windows)]
            if ::simple_tauri::check_mutex(#name) {
                return;
            }
        },
        None => quote! {
            #[cfg(windows)]
            if ::simple_tauri::check_mutex(::std::env!("CARGO_PKG_NAME")) {
                return;
            }
        },
    }
    .into()
}

/// 启动应用：等价 simple_tray::run(tauri::generate_context!())
/// 由宏在调用方 crate 展开，读取该 crate 的 tauri.conf.json
#[proc_macro]
pub fn run(input: TokenStream) -> TokenStream {
    if !input.is_empty() {
        panic!("run!: 不接受任何参数, 直接写 run!()");
    }
    quote! {
        ::simple_tauri::simple_tray::run(::tauri::generate_context!());
    }
    .into()
}
