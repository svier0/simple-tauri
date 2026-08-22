use proc_macro::TokenStream;
use quote::quote;

pub fn start_impl(input: TokenStream) -> TokenStream {
    let input = proc_macro2::TokenStream::from(input);
    quote! {
        ::simple_tauri::simple_serve::start(&format!(#input))
    }
    .into()
}
