use proc_macro::TokenStream;
use quote::quote;

pub fn set_start_cmd_impl(input: TokenStream) -> TokenStream {
    let input = proc_macro2::TokenStream::from(input);
    quote! {
        ::simple_tauri::simple_serve::set_start_cmd(&format!(#input))
    }
    .into()
}
