use au2_rust_server::{NoteStore, app};
use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use serde_json::{Value, json};
use tower::ServiceExt;

async fn send(app: &Router, method: &str, uri: &str, body: Option<Value>) -> (StatusCode, Value) {
    let request = Request::builder().method(method).uri(uri);
    let request = match body {
        Some(json) => request
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json.to_string())),
        None => request.body(Body::empty()),
    }
    .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    };
    (status, json)
}

fn test_app() -> Router {
    app(NoteStore::default(), None)
}

#[tokio::test]
async fn health_reports_ok() {
    let (status, body) = send(&test_app(), "GET", "/api/health", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn create_list_and_delete_a_note() {
    let app = test_app();

    let (status, note) = send(
        &app,
        "POST",
        "/api/notes",
        Some(json!({ "title": "  Hello, Rust!  ", "body": "From Aurelia." })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(note["title"], "Hello, Rust!");
    assert_eq!(note["slug"], "hello-rust");

    let (status, notes) = send(&app, "GET", "/api/notes", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(notes.as_array().unwrap().len(), 1);

    let uri = format!("/api/notes/{}", note["id"]);
    let (status, _) = send(&app, "DELETE", &uri, None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, _) = send(&app, "DELETE", &uri, None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn invalid_note_returns_field_errors() {
    let (status, body) = send(
        &test_app(),
        "POST",
        "/api/notes",
        Some(json!({ "title": "   ", "body": "" })),
    )
    .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["errors"][0]["field"], "title");
}

#[tokio::test]
async fn unknown_api_route_is_a_json_404() {
    let (status, body) = send(&test_app(), "GET", "/api/nope", None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["message"], "No such API route.");
}

#[tokio::test]
async fn client_routes_fall_back_to_index_html() {
    let dir = std::env::temp_dir().join(format!("au2-rust-static-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("index.html"), "<my-app></my-app>").unwrap();

    let request = Request::get("/notes/some-slug")
        .body(Body::empty())
        .unwrap();
    let response = app(NoteStore::default(), Some(&dir))
        .oneshot(request)
        .await
        .unwrap();
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    std::fs::remove_dir_all(&dir).unwrap();

    assert_eq!(status, StatusCode::OK);
    assert_eq!(&body[..], b"<my-app></my-app>");
}
