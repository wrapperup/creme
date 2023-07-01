use std::path::{Path, PathBuf};

use lightningcss::{
    bundler::{Bundler, FileProvider},
    dependencies::DependencyOptions,
    error::{Error as LightningCssError, PrinterErrorKind},
    stylesheet::{ParserOptions, PrinterOptions},
    targets::Targets,
};
use once_cell::sync::Lazy;
use path_absolutize::Absolutize;
use thiserror::Error;

use crate::MANIFEST;

#[derive(Error, Debug)]
pub enum BundleError {
    #[error("bundler error: {0}")]
    Bundler(String),
    #[error("browsers error: {0}")]
    Browsers(#[from] browserslist::Error),
    #[error("print error: {0}")]
    Print(#[from] LightningCssError<PrinterErrorKind>),
}

static FILE_PROVIDER: Lazy<FileProvider> = Lazy::new(FileProvider::new);

// TODO: omg this is so bad
fn resolve_url(dep_url: &String, src_path: &Path, assets_dir: &PathBuf) -> String {
    if dep_url.starts_with("https://") || dep_url.starts_with("http://") {
        return dep_url.clone();
    }

    let full_asset_path = std::fs::canonicalize(assets_dir).unwrap();

    let full_path = full_asset_path
        .join(src_path.strip_prefix(assets_dir).unwrap().parent().unwrap())
        .join(dep_url);
    let binding = full_path.absolutize().unwrap();
    let url = binding.strip_prefix(full_asset_path).unwrap();

    let url = url.to_str().unwrap().replace('\\', "/");

    MANIFEST
        .lock()
        .unwrap()
        .assets
        .get(&url)
        .cloned()
        .unwrap()
}

pub(crate) fn process_css(
    path: &Path,
    parser_options: ParserOptions,
    targets: impl Into<Targets>,
    assets_dir: &PathBuf,
) -> String {
    // let mut bundler = Bundler::new_with_at_rule_parser(&*FILE_PROVIDER, None, parser_options);
    let mut bundler = Bundler::new(&*FILE_PROVIDER, None, parser_options);
    let stylesheet = bundler.bundle(path).unwrap();

    let css = stylesheet
        .to_css(PrinterOptions {
            minify: true,
            targets: targets.into(),
            analyze_dependencies: Some(DependencyOptions {
                remove_imports: false,
            }),
            ..PrinterOptions::default()
        })
        .unwrap();

    let mut code = css.code;

    css.dependencies.unwrap().iter().for_each(|dep| {
        let (placeholder, path, url) = match dep {
            lightningcss::dependencies::Dependency::Url(url_dep) => {
                (&url_dep.placeholder, &url_dep.loc.file_path, &url_dep.url)
            }
            lightningcss::dependencies::Dependency::Import(import_dep) => {
                (&import_dep.placeholder, &import_dep.loc.file_path, &import_dep.url)
            }
        };

        let resolved_path = resolve_url(url, &PathBuf::from(path), assets_dir);

        // TODO: Probably need to include the / in the manifest
        code = code.replace(placeholder, &format!("/{resolved_path}"));
    });

    code
}
