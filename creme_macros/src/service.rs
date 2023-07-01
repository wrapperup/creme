use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;

pub fn service(input: TokenStream) -> syn::Result<TokenStream> {
    let quoted = if let Ok(env) = std::env::var("CREME_RELEASE_MODE") {
        if env == "release" {
            input
        } else {
            input
        }
    } else {
        return Err(syn::Error::new(
            Span::call_site(),
            "CREME_RELEASE_MODE not set. Usually this means that you are not using creme_bundler in your build script, or it didn't bundle."
        ));
    };

    Ok(quoted)
}
