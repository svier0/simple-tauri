use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Expr, Token};
use syn::parse::{Parse, ParseStream};

struct IpcResultInput {
    args: Vec<Expr>,
}

impl Parse for IpcResultInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut args = Vec::new();
        while !input.is_empty() {
            args.push(input.parse::<Expr>()?);
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(IpcResultInput { args })
    }
}

pub fn ipc_result_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as IpcResultInput);

    match input.args.len() {
        1 => {
            let param = &input.args[0];
            let expanded = quote! {
                match #param {
                    Ok(v) => {
                        let data: serde_json::Value = v.into();
                        serde_json::json!({"code": 0, "msg": "", "data": data})
                    },
                    Err(e) => serde_json::json!({"code": 1, "msg": e.to_string(), "data": serde_json::Value::Null}),
                }
            };
            expanded.into()
        }
        2 => {
            let code = &input.args[0];
            let msg = &input.args[1];
            let expanded = quote! {
                {
                    let data: serde_json::Value = ().into();
                    serde_json::json!({"code": #code, "msg": #msg, "data": data})
                }
            };
            expanded.into()
        }
        3 => {
            let code = &input.args[0];
            let msg = &input.args[1];
            let data = &input.args[2];
            let expanded = quote! {
                {
                    let data: serde_json::Value = (#data).into();
                    serde_json::json!({"code": #code, "msg": #msg, "data": data})
                }
            };
            expanded.into()
        }
        _ => {
            syn::Error::new_spanned(
                quote! { ipc_result },
                "ipc_result! 需要 1 个参数(Result) 或 3 个参数(code, msg, data)",
            )
            .to_compile_error()
            .into()
        }
    }
}
