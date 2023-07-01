use std::{collections::HashMap, env, fs::File, path::PathBuf};

use once_cell::sync::Lazy;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use serde::Deserialize;
use syn::{
    parse::{Parse, ParseStream},
    LitStr,
};

#[derive(Deserialize)]
struct Manifest {
    assets: HashMap<String, String>,
}

static MANIFEST: Lazy<Manifest> = Lazy::new(|| {
    let manifest_dir = PathBuf::from(env::var("CREME_MANIFEST").expect("CREME_MANIFEST not set"));

    let file_reader = File::open(manifest_dir).expect("Failed to open manifest file");
    let manifest: Manifest =
        serde_json::from_reader(file_reader).expect("Failed to parse manifest file");

    manifest
});

struct StaticInput {
    pub path: String,
}

impl Parse for StaticInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path = input.parse::<LitStr>()?.value();
        Ok(Self { path })
    }
}

pub fn asset(input: TokenStream) -> syn::Result<TokenStream> {
    let StaticInput { path } = syn::parse::<StaticInput>(input)?;

    if env::var("CREME_MANIFEST").is_err() {
    let path = "assets/".to_string() + &path;

        return Ok(quote! {
            #path
        }
        .into());
    }

    let asset_path = MANIFEST.assets.get(&path).ok_or(syn::Error::new(
        Span::call_site(),
        format!("Asset \"{path}\" not found in manifest"),
    ))?;

    Ok(quote! {
        #asset_path
    }
    .into())
}
