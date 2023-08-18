use std::path::PathBuf;

use axum::{routing::get, Router};
use creme::asset;
use maud::{html, Markup};
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(index_handler))
        .fallback_service(CremeService::new(
            PathBuf::from(env!("CREME_ASSETS_DIR")),
            PathBuf::from(env!("CREME_PUBLIC_DIR"))
        ));

    #[cfg(debug_assertions)]
    let app = app.layer(
        tower_livereload::LiveReloadLayer::new()
            .reload_interval(std::time::Duration::from_millis(50)),
    );

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn index_handler() -> Markup {
    html! {
        html lang="en" {
            head {
                meta charset="UTF-8";
                title { "Rust + Creme + Maud" }
                link rel="stylesheet" href=(asset!("css/style.css"));
            }
            body {
                section {
                    div style="display: flex; align-items: center; gap: 1rem" {
                        h1 style="font-size: 7rem; text-align: center" { "Hello!" }
                    }
                    h2 { "Bye world!" }
                    p { "This is a paragraph." }
                }
                section {
                    h2 { "Here's a picture of a cat." }
                    p { "It's a cute cat." }
                }
                section class="full-width" {
                    img src=(asset!("img/cat.jpeg")) alt="el gato";
                    p class="caption" { "This isn't my cat, but it's a cute cat." }
                }
                section {
                    h2 { "Here's a list of things I like:" }
                    p {
                        "These are in no particular order, except for cats."
                        div style="margin-left:-1rem" {
                            ul {
                                li { "Cats - any kind of cat is a good cat!" }
                                li { "Programming - This is a programming website!" }
                                li { "Music - I like to listen to it!" }
                            }
                        }
                    }
                }
                section {
                    h2 { "Here's a list of things I don't like:" }
                    p { "These are in order of how much I don't like them."
                        div style="margin-left:-1rem" {
                            ol {
                                li { "Wasps - no good, not cool!" }
                                li { "Spiders - very scary, but friendly!" }
                                li { "Snakes - cool, but scary!" }
                            }
                        }
                    }
                }
                section {
                    h2 { "Here's a table of things I like and don't like:" }
                    p { "I like table formatting. Deal with it."
                        table {
                            thead {
                                tr {
                                    th { "Like" }
                                    th { "Don't Like" }
                                }
                            }
                            tbody {
                                tr {
                                    td { "Cats" }
                                    td { "Wasps" }
                                }
                                tr {
                                    td { "Programming" }
                                    td { "Spiders" }
                                }
                                tr {
                                    td { "Music" }
                                    td { "Snakes" }
                                }
                            }
                        }
                    }
                }
                section {
                    h2 { "Here's a lot of text:" }
                    p {
                        "Lorem ipsum dolor sit amet, consectetur adipiscing elit.
                        Nullam euismod, nisl nec aliquam ultricies, nunc nisl
                        ultricies nunc, nec aliquam nunc nisl nec nisi. Donec
                        euismod, nisl nec aliquam ultricies, nunc nisl ultricies"
                    }
                    p {
                        "nunc, nec aliquam nunc nisl nec nisi. Donec euismod, nisl
                        nec aliquam ultricies, nunc nisl ultricies nunc, nec
                        aliquam nunc nisl nec nisi. Donec euismod, nisl nec
                        aliquam ultricies, nunc nisl ultricies nunc, nec aliquam"
                    }
                    p {
                        "nunc nisl nec nisi. Donec euismod, nisl nec aliquam
                        ultricies, nunc nisl ultricies nunc, nec aliquam nunc
                        nisl nec nisi. Donec euismod, nisl nec aliquam ultricies,
                        nunc nisl ultricies nunc, nec aliquam nunc nisl nec nisi."
                    }
                }
                section class="full-width" {
                    img src=(asset!("img/cat.jpeg")) alt="el gato";
                    p class="caption" { "Wait. This is the same cat as before!" }
                }
                section {
                    p { "Here's a link to " a href="https://www.google.com" { "Google" } }
                    p { "That's all for now!" }
                }
            }
        }
    }
}
