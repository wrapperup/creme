use std::path::PathBuf;

use askama::Template;
use askama_axum::IntoResponse;
use axum::{routing::get, Router, handler::HandlerWithoutStateExt, http::StatusCode};
use creme::asset;
use creme::services::CremeService;
use tower_http::services::ServeDir;

use creme::embed::{EmbeddedAssets, EmbeddedAsset};

#[tokio::main]
async fn main() {
    println!("asset 1: {:?}", ASSETS.get(0));
    let app = Router::new()
        .route("/", get(index_handler))
        .fallback_service(CremeService::new(
            PathBuf::from(env!("CREME_ASSETS_DIR")),
            PathBuf::from(env!("CREME_PUBLIC_DIR"))
        ));

    // Uncomment this to disable hot reloading in release mode.
    // #[cfg(debug_assertions)]
    let app = app.layer(
        tower_livereload::LiveReloadLayer::new()
            .reload_interval(std::time::Duration::from_millis(100)),
    );

    println!("Listening on http://localhost:3000");

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {}

async fn index_handler() -> impl IntoResponse {
    IndexTemplate {}
}

#[derive(Template)]
#[template(path = "404.html")]
struct NotFoundTemplate {}

async fn not_found_handler() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, NotFoundTemplate {})
}
