use proc_macro::TokenStream;

mod asset;
mod service;

/// A macro that reads from the creme-manifest.json file and returns the path to the asset.
/// # Example
/// ```rust
/// use creme::asset;
///
/// // Transforms "my_asset.png" -> "assets/my_asset-[hash].png"
/// let path = asset!("my_asset.png");
/// ```
#[proc_macro]
pub fn asset(input: TokenStream) -> TokenStream {
    match asset::asset(input) {
        Ok(ts) => ts,
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn service(input: TokenStream) -> TokenStream {
    match service::service(input) {
        Ok(ts) => ts,
        Err(e) => e.to_compile_error().into(),
    }
}
