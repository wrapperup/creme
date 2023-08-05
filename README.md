# crème 🍦

>⚠️ creme is in an experimental state. Use with caution!

Creme is a simple, opinionated build-time asset bundler tool for 
Rust websites, complete with compile-time checks.

It's designed to be used with monolithic templated web apps, but it can
be used with front-end frameworks like Dioxus and Leptos. Creme exists in
two parts:

* Tower middleware. This handles serving your assets in both dev
mode and release mode.

* Build-time bundler. Creme will bundle your assets when building for
release. It provides a handy `asset!()` macro to reference your assets and
ensures they exist at compile time.

## Features
* 🔥 Fast Dev-Mode Iteration
* 📁 Static File Handling
* 🔎 Cache Busting
* ⚡ CSS Bundling With LightningCSS

## Usage

[Check out the examples here](/examples)

Creme uses a build script to generate the output assets. Creme has
an opinionated, but customizable default configuration.

It can be used simply with:

```rust
use creme_bundler::{Creme, CremeResult};

fn main() -> CremeResult<()> {
    // Recommended to include. Creme will setup the rest.
    println!("cargo:rerun-if-changed=build.rs");

    Creme::new()
        .out_dir_build_rs()? // builds the dist in OUT_DIR
        // .out_dir("dist")? // builds the dist in the `dist` folder
        .recommended()?
        .bundle()
}
```

Creme's `recommended` setting expects an "assets" and "public" directory. It will also use dev mode when compiling in debug mode.

```
my_website/
├── Cargo.toml
├── src/
│   └── main.rs
│
├── public/
│   ├── robots.txt
│   └── favicon.ico
└── assets/
    └── css/
        ├── style.css
        └── modules/
            ├── _mod1.css
            └── _mod2.css
```

The "assets" directory will be transformed (bundled, hashed, etc) by
Creme. "public" files are copied without any modifications, and
retain the file structure.

Any files that start with an underscore will not be included in the
final release output, but are processed. This is great for module
files, like CSS imports.

To reference assets in your code, use the included `creme::assets!`
macro to get the URL. Presto!

```rust
use creme::asset;

asset!("css/style.css");
// Becomes "/assets/style-[hash].css" in release mode
// Or "/assets/css/style.css" in dev mode.
```

This can be used easily in a compile-time templating context,
such as rsx, Leptos, Dioxus, Askama, Maud... you name it.

For example, in Maud:
```rust
use creme::asset;

html! { 
    head {
        title { "Hello Creme + Maud!" }
        link rel="stylesheet" href=(asset!("css/style.css"));
    }
    ...
    img src=(asset!("img/cat.jpeg"));
}
```

For more, [see here for examples](/examples)

## Development Mode

During development, Creme doesn't actually transform any files. Instead,
files are served directly from disk. The benefit is that doesn't require
a recompilation for any asset change, keeping iteration times low and
fast!

By default, Creme will run in development mode when running in debug mode.
For CSS, it is recommended to use a browser that supports experimental
CSS features (nesting, custom-media) that LightningCSS also supports.
This is because Creme will directly load your CSS.

## Release Mode

When building for release, Creme will build and bundle your assets.
Filenames are also given a hash (in the format of `filename-[hash].ext`)
to both prevent collisions when squashing the files into the outputted
"assets" directory, and for cache busting.

CSS is also optimized, minified, and expanded for widespread browser support,
thanks to LightningCSS.
