use std::sync::{Arc, RwLock};

use au2_rust_core::{FieldError, Note, NoteDraft, slugify};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get},
};
use serde::Serialize;

pub fn routes() -> Router<NoteStore> {
    Router::new()
        .route("/notes", get(list).post(create))
        .route("/notes/{id}", delete(remove))
}

/// In-memory note storage. Swap this for a database when you need one; the
/// handlers only depend on these three methods.
#[derive(Clone, Default)]
pub struct NoteStore {
    inner: Arc<RwLock<Notes>>,
}

#[derive(Default)]
struct Notes {
    next_id: u64,
    items: Vec<Note>,
}

impl NoteStore {
    pub fn list(&self) -> Vec<Note> {
        self.inner
            .read()
            .expect("note store poisoned")
            .items
            .clone()
    }

    /// Validates with the shared rules, then stores the trimmed draft.
    pub fn insert(&self, draft: NoteDraft) -> Result<Note, Vec<FieldError>> {
        let errors = draft.validate();
        if !errors.is_empty() {
            return Err(errors);
        }

        let NoteDraft { title, body } = draft.trimmed();
        let mut notes = self.inner.write().expect("note store poisoned");
        notes.next_id += 1;
        let note = Note {
            id: notes.next_id,
            slug: slugify(&title),
            title,
            body,
        };
        notes.items.push(note.clone());
        Ok(note)
    }

    pub fn remove(&self, id: u64) -> bool {
        let mut notes = self.inner.write().expect("note store poisoned");
        let before = notes.items.len();
        notes.items.retain(|note| note.id != id);
        notes.items.len() != before
    }
}

async fn list(State(store): State<NoteStore>) -> Json<Vec<Note>> {
    Json(store.list())
}

async fn create(
    State(store): State<NoteStore>,
    Json(draft): Json<NoteDraft>,
) -> Result<(StatusCode, Json<Note>), ApiError> {
    let note = store.insert(draft).map_err(ApiError::Invalid)?;
    Ok((StatusCode::CREATED, Json(note)))
}

async fn remove(
    State(store): State<NoteStore>,
    Path(id): Path<u64>,
) -> Result<StatusCode, ApiError> {
    if store.remove(id) {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound)
    }
}

enum ApiError {
    Invalid(Vec<FieldError>),
    NotFound,
}

#[derive(Serialize)]
struct ErrorBody {
    message: &'static str,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    errors: Vec<FieldError>,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message, errors) = match self {
            Self::Invalid(errors) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "The note is not valid.",
                errors,
            ),
            Self::NotFound => (StatusCode::NOT_FOUND, "Note not found.", Vec::new()),
        };
        (status, Json(ErrorBody { message, errors })).into_response()
    }
}
