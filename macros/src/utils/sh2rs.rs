use proc_macro::TokenStream;
use quote::quote;

pub fn sh2rs_impl(input: TokenStream) -> TokenStream {
    let input = proc_macro2::TokenStream::from(input);
    quote! {
        ::simple_tauri::utils::sh2rs::sh2rs(&format!(#input))
    }
    .into()
}
