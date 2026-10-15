mod notes;

use std::path::Path;

use axum::{Json, Router, http::StatusCode, routing::get};
use serde::Serialize;
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

pub use notes::NoteStore;

/// Builds the application router.
///
/// Everything under `/api` is JSON. When `static_dir` points at a built
/// frontend (`web/dist`), every other path is served from it, with
/// `index.html` as the fallback so client-side routes survive a reload.
pub fn app(store: NoteStore, static_dir: Option<&Path>) -> Router {
    let api = Router::new()
        .route("/health", get(health))
        .merge(notes::routes())
        .fallback(api_not_found)
        .with_state(store);

    let router = Router::new().nest("/api", api);

    let router = match static_dir {
        Some(dir) => router
            .fallback_service(ServeDir::new(dir).fallback(ServeFile::new(dir.join("index.html")))),
        None => router,
    };

    router.layer(TraceLayer::new_for_http())
}

#[derive(Serialize)]
struct Health {
    status: &'static str,
    version: &'static str,
}

async fn health() -> Json<Health> {
    Json(Health {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
    })
}

#[derive(Serialize)]
struct Message {
    message: &'static str,
}

async fn api_not_found() -> (StatusCode, Json<Message>) {
    (
        StatusCode::NOT_FOUND,
        Json(Message {
            message: "No such API route.",
        }),
    )
}
