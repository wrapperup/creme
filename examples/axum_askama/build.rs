use creme_bundler::{Creme, CremeResult};

fn main() -> CremeResult<()> {
    println!("cargo:rerun-if-changed=build.rs");

    // Creme is a build-time asset bundler. It takes a directory of assets and
    // bundles them into a Rust module that can be used at runtime.
    Creme::new()
        // .out_dir("./dist")
        .out_dir_build_rs()?
        // .recommended()?
        .default_config()?
        .release()
        .bundle()
}
