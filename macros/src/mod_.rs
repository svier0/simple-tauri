use proc_macro::TokenStream;
use proc_macro2::Literal;
use quote::quote;
use syn::{parse_macro_input, LitStr};

pub fn mutex_impl(input: TokenStream) -> TokenStream {
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
