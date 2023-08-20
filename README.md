# crème 🍦

>⚠️ creme is in an experimental state. Use with caution!

Creme is a simple, opinionated build-time static asset tool for 
server-side Rust websites, complete with compile-time checks and
a fast dev server for quick iteration.

## Features
* 🔥 Fast Dev-Mode Server
* 📁 Static File Handling
* 🔎 Cache Busting
* ⚡ CSS Bundling With LightningCSS

## Usage

[Check out the examples here](/examples)

Create a `public` and `assets` folder in your project's manifest.
Public files are copied over without any modifications, asset files
are hashed, minified, and optimized.

An example project may look like this:

```
my_website/
├── Cargo.toml
├── src/
│   ├── main.rs
│   └── ...
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

Configure Creme in your `build.rs` script, using the included builder.

Creme lets you either embed assets into the binary for quick and easy deployment,
or to a directory of your choice for deploying to a CDN.

```rust
use creme_bundler::{Creme, CremeResult};

fn main() -> CremeResult<()> {
    // Recommended to include. Creme will setup the rest.
    println!("cargo:rerun-if-changed=build.rs");

    Creme::new()
        .out_dir("dist")? // Outputs assets to the `dist` directory, for CDN deployment.
        // .embedded()? // Or prepare assets to be embedded into the binary and served directly.
        .recommended()? // Reads from `public` and `assets` folder.
        .bundle()
}
```

In your Rust code, reference an asset's URL:

```rust
use creme::asset;

asset!("css/style.css");
// Becomes "/assets/style-[hash].css" in release mode
// Or "/assets/css/style.css" in dev mode.
```

Or directly in your template engine of choice:

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

Optionally, use the built-in tower `creme::service!()` macro. This handles
creating and setting up the dev server service.

For example, with Axum:

```rust
let app = Router::new()
    .route("/", get(index_handler))
    .fallback_service(
        creme::service!()
            .fallback(not_found_handler.into_service())
    );
```

For more, [see here for examples](/examples)
