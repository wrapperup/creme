use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;

pub fn service(input: TokenStream) -> syn::Result<TokenStream> {
    let quoted = if let Ok(env) = std::env::var("CREME_RELEASE_MODE") {
        if env == "release" {
            // TODO: Not implemented yet. This handles embedded assets.
            quote! {
                ::creme::services::CremeDevService::new(
                    ::std::path::PathBuf::from(::core::env!("CREME_ASSETS_DIR")),
                    ::std::path::PathBuf::from(::core::env!("CREME_PUBLIC_DIR"))
                )
            }
        } else {
            quote! {
                ::creme::services::CremeDevService::new(
                    ::std::path::PathBuf::from(::core::env!("CREME_ASSETS_DIR")),
                    ::std::path::PathBuf::from(::core::env!("CREME_PUBLIC_DIR"))
                )
            }
        }
    } else {
        return Err(syn::Error::new(
            Span::call_site(),
            "CREME_RELEASE_MODE not set. Usually this means that you are not using creme_bundler in your build script, or it didn't bundle."
        ));
    };

    Ok(quoted.into())
}
