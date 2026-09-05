use quote::quote;
use syn::LitStr;

pub fn get_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input_str = input.to_string();
    let parts: Vec<&str> = input_str.splitn(2, ',').map(|s| s.trim()).collect();

    let key: LitStr = syn::parse_str(parts[0]).expect("get!: 第一个参数必须是字符串字面量");

    let expanded = if parts.len() < 2 || parts[1].is_empty() {
        quote! { ::simple_tauri::config::get(#key, serde_json::Value::Null) }
    } else {
        let default: proc_macro2::TokenStream = syn::parse_str(parts[1])
            .expect("get!: 第二个参数解析失败");
        quote! {
            ::simple_tauri::config::get(#key, (#default).into())
        }
    };
    expanded.into()
}

pub fn get_or_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input_str = input.to_string();
    let parts: Vec<&str> = input_str.splitn(2, ',').map(|s| s.trim()).collect();

    let key: LitStr = syn::parse_str(parts[0]).expect("get_or!: 第一个参数必须是字符串字面量");
    let default_str = parts.get(1).map(|s| *s).unwrap_or("");

    let default_tokens: proc_macro2::TokenStream = syn::parse_str(default_str)
        .expect("get_or!: 第二个参数解析失败");

    let expanded = quote! {
        {
            let v = ::simple_tauri::config::get(#key, serde_json::Value::Null);
            <_ as ::simple_tauri::config::ConfigDefault>::from_value(&v).unwrap_or(#default_tokens)
        }
    };
    expanded.into()
}
