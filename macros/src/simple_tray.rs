use proc_macro2::Literal;
use quote::quote;

/// 判断值是否以 "rust:" 开头，是则返回 Rust 表达式，否则返回 None
fn parse_rust_expr(s: &str) -> Option<&str> {
    s.strip_prefix("rust:")
}

/// 处理 JSON 值：如果是字符串且以 "rust:" 开头，返回 Rust 表达式 token；否则按原始类型处理
fn value_to_token(v: &serde_json::Value) -> proc_macro2::TokenStream {
    match v {
        serde_json::Value::String(s) => {
            if let Some(expr) = parse_rust_expr(s) {
                expr.parse().expect("run!: rust: 表达式解析失败")
            } else {
                quote!(#s)
            }
        }
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                quote!(#i)
            } else if let Some(f) = n.as_f64() {
                quote!(#f)
            } else {
                quote!(0)
            }
        }
        serde_json::Value::Bool(b) => quote!(#b),
        serde_json::Value::Null => quote!(""),
        _ => quote!(""),
    }
}

pub fn run_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    if !input.is_empty() {
        panic!("run!: 不接受任何参数, 直接写 run!()");
    }

    // 读取 jsonc 文件
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("run!: CARGO_MANIFEST_DIR 未设置");
    let jsonc_path = std::path::Path::new(&manifest_dir)
        .join("src")
        .join("simple_tauri.jsonc");
    let jsonc_str = std::fs::read_to_string(&jsonc_path)
        .unwrap_or_else(|e| panic!("run!: 读取 {jsonc_path:?} 失败: {e}"));
    
    // 解析 json5（支持注释）
    let json: serde_json::Value = json5::from_str(&jsonc_str)
        .unwrap_or_else(|e| panic!("run!: JSONC 解析失败: {e}"));

    let mut tokens = Vec::new();

    // 1. generate_context
    tokens.push(quote! {
        let __context = ::tauri::generate_context!();
    });

    // 2. set_product_name
    tokens.push(quote! {
        ::simple_tauri::simple_tray::set_product_name(
            __context.config().product_name.clone().unwrap_or_default()
        );
    });

    // 3. mutex
    if json.get("mutex").and_then(|v| v.as_bool()).unwrap_or(false) {
        tokens.push(quote! {
            ::simple_tauri::mutex!();
        });
    }

    // 4. on_tauri_before hook
    if let Some(hooks) = json.get("hooks") {
        if let Some(before) = hooks.get("on_tauri_before").and_then(|v| v.as_str()) {
            if before.starts_with('@') {
                let func_name = &before[1..];
                let func_ident: proc_macro2::TokenStream = func_name.parse()
                    .expect("run!: on_tauri_before 函数名解析失败");
                tokens.push(quote! {
                    #func_ident();
                });
            }
        }
    }

    // 5. ipc_cmds - 扫描模块源码找 #[tauri::command] 函数
    if let Some(ipc_cmds) = json.get("ipc_cmds").and_then(|v| v.as_array()) {
        let modules: Vec<String> = ipc_cmds.iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();
        
        if !modules.is_empty() {
            let mut all_paths: Vec<proc_macro2::TokenStream> = Vec::new();
            
            for module in &modules {
                let ipc_path = std::path::Path::new(&manifest_dir)
                    .join("src")
                    .join(format!("{module}.rs"));
                let src = std::fs::read_to_string(&ipc_path).unwrap_or_else(|e| {
                    panic!("run!: 读取 {ipc_path:?} 失败: {e}")
                });
                
                let file: syn::File = syn::parse_file(&src)
                    .unwrap_or_else(|e| panic!("run!: {module}.rs 解析失败: {e}"));
                
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
                            let module_ident = syn::Ident::new(module, proc_macro2::Span::call_site());
                            let func_name = &f.sig.ident;
                            all_paths.push(quote!(#module_ident::#func_name));
                        }
                    }
                }
            }
            
            if all_paths.is_empty() {
                tokens.push(quote! {
                    ::simple_tauri::simple_tray::set_ipc_cmds(|_| false);
                });
            } else {
                tokens.push(quote! {
                    ::simple_tauri::simple_tray::set_ipc_cmds(
                        tauri::generate_handler![#(#all_paths),*]
                    );
                });
            }
        }
    }

    // 6. window_list
    if let Some(window_list) = json.get("window_list").and_then(|v| v.as_array()) {
        let mut window_items = Vec::new();
        for row in window_list {
            let row = row.as_array().expect("run!: window_list 每项必须是数组");
            
            let id = row.first().and_then(|v| v.as_str()).unwrap_or_default();
            let title = row.get(1).and_then(|v| v.as_str()).unwrap_or_default();
            
            // url: null 时用默认值
            let url_token = match row.get(2) {
                Some(serde_json::Value::Null) => {
                    let default_url = format!("{id}.html");
                    quote!(#default_url)
                }
                Some(v) => value_to_token(v),
                None => {
                    let default_url = format!("{id}.html");
                    quote!(#default_url)
                }
            };
            
            // width: null 时用默认值 800.0
            let width_token = match row.get(3) {
                Some(serde_json::Value::Null) => quote!(800.0),
                Some(v) => value_to_token(v),
                None => quote!(800.0),
            };
            
            // height: null 时用默认值 540.0
            let height_token = match row.get(4) {
                Some(serde_json::Value::Null) => quote!(540.0),
                Some(v) => value_to_token(v),
                None => quote!(540.0),
            };
            
            // decorations: null 时用默认值 true
            let decorations_token = match row.get(5) {
                Some(serde_json::Value::Null) => quote!(true),
                Some(v) => value_to_token(v),
                None => quote!(true),
            };

            let id = Literal::string(id);
            let title = Literal::string(title);

            window_items.push(quote! {
                ::simple_tauri::simple_tray::WindowConfig {
                    id: #id.to_string(),
                    title: #title.to_string(),
                    url: #url_token.to_string(),
                    width: #width_token as f64,
                    height: #height_token as f64,
                    decorations: #decorations_token as bool,
                }
            });
        }
        tokens.push(quote! {
            ::simple_tauri::simple_tray::set_window_list(vec![#(#window_items),*]);
        });
    }

    // 7. tray_menu
    if let Some(tray_menu) = json.get("tray_menu").and_then(|v| v.as_array()) {
        let mut menu_items = Vec::new();
        for row in tray_menu {
            let row = row.as_array().expect("run!: tray_menu 每项必须是数组");
            let id = row.first().and_then(|v| v.as_str()).unwrap_or_default();
            let label = row.get(1).and_then(|v| v.as_str()).unwrap_or_default();
            let cb_str = row.get(2).and_then(|v| v.as_str()).unwrap_or_default();

            if id.is_empty() {
                menu_items.push(quote! {
                    ("", "", None)
                });
                continue;
            }

            let id = Literal::string(id);
            let label = Literal::string(label);

            if cb_str.is_empty() {
                menu_items.push(quote! {
                    (#id, #label, None)
                });
            } else {
                let cb_str_clean = cb_str.trim_start_matches('@');
                let cb_ident: proc_macro2::TokenStream = cb_str_clean.parse()
                    .expect("run!: tray_menu 回调函数名解析失败");
                menu_items.push(quote! {
                    (#id, #label, Some(#cb_ident as fn()))
                });
            }
        }
        tokens.push(quote! {
            ::simple_tauri::simple_tray::set_tray_menu(&[#(#menu_items),*]);
        });
    }

    // 8. hooks (on_tray_before, on_tray_after, on_quit)
    if let Some(hooks) = json.get("hooks") {
        let before = hooks.get("on_tray_before").and_then(|v| v.as_str());
        let after = hooks.get("on_tray_after").and_then(|v| v.as_str());
        let quit = hooks.get("on_quit").and_then(|v| v.as_str());

        let hook_to_token = |opt: Option<&str>| -> proc_macro2::TokenStream {
            match opt {
                Some(s) if s == "_" => quote!(None),
                Some(s) => {
                    let clean = s.trim_start_matches('@');
                    let ident: proc_macro2::TokenStream = clean.parse().unwrap();
                    quote!(Some(#ident as fn() -> ::std::result::Result<(), ::std::string::String>))
                }
                None => quote!(None),
            }
        };

        let before_token = hook_to_token(before);
        let after_token = hook_to_token(after);
        let quit_token = hook_to_token(quit);

        tokens.push(quote! {
            ::simple_tauri::simple_tray::set_hooks(#before_token, #after_token, #quit_token);
        });
    }

    // 9. server.package (set_pkg)
    if let Some(server) = json.get("server") {
        if let Some(package) = server.get("package") {
            let pkg_type = package.get("type").and_then(|v| v.as_str()).unwrap_or("");
            let pkg_name = package.get("name").and_then(|v| v.as_str()).unwrap_or("");
            tokens.push(quote! {
                ::simple_tauri::simple_serve::set_pkg(#pkg_type, #pkg_name);
            });
        }
    }

    // 10. run
    tokens.push(quote! {
        ::simple_tauri::simple_tray::run(__context);
    });

    let expanded = quote! {
        #(#tokens)*
    };
    expanded.into()
}
