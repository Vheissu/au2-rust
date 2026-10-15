//! Domain types and rules shared by the server and the browser.
//!
//! The server links this crate natively. The browser gets the same code
//! compiled to WebAssembly through `crates/wasm`, so a draft that passes
//! validation in the form is guaranteed to pass on the server too.

use serde::{Deserialize, Serialize};

#[cfg(feature = "typescript")]
use tsify::Tsify;

pub const TITLE_MAX_CHARS: usize = 80;
pub const BODY_MAX_CHARS: usize = 2_000;

/// What a client sends to create a note.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(Tsify))]
pub struct NoteDraft {
    pub title: String,
    pub body: String,
}

/// A stored note, as returned by the API.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(Tsify))]
pub struct Note {
    pub id: u64,
    pub title: String,
    pub body: String,
    pub slug: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(Tsify))]
#[serde(rename_all = "lowercase")]
pub enum Field {
    Title,
    Body,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(Tsify))]
pub struct FieldError {
    pub field: Field,
    pub message: String,
}

/// The character limits a draft is validated against.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct Limits {
    pub title_max_chars: usize,
    pub body_max_chars: usize,
}

pub const LIMITS: Limits = Limits {
    title_max_chars: TITLE_MAX_CHARS,
    body_max_chars: BODY_MAX_CHARS,
};

impl NoteDraft {
    /// Returns a copy with surrounding whitespace removed.
    pub fn trimmed(&self) -> Self {
        Self {
            title: self.title.trim().to_owned(),
            body: self.body.trim().to_owned(),
        }
    }

    /// Checks the draft against the note rules. An empty list means valid.
    ///
    /// Lengths are counted in Unicode scalar values, not UTF-16 code units,
    /// so "🦀" is one character here even though JavaScript reports
    /// `"🦀".length === 2`. Running this exact function on both sides is what
    /// keeps the browser and the server from disagreeing.
    pub fn validate(&self) -> Vec<FieldError> {
        let mut errors = Vec::new();
        let title = self.title.trim();
        let body = self.body.trim();

        if title.is_empty() {
            errors.push(FieldError::new(Field::Title, "Title is required."));
        } else if char_count(title) > TITLE_MAX_CHARS {
            errors.push(FieldError::new(
                Field::Title,
                format!("Title must be {TITLE_MAX_CHARS} characters or fewer."),
            ));
        } else if slugify(title).is_empty() {
            errors.push(FieldError::new(
                Field::Title,
                "Title needs at least one letter or number.",
            ));
        }

        if char_count(body) > BODY_MAX_CHARS {
            errors.push(FieldError::new(
                Field::Body,
                format!("Body must be {BODY_MAX_CHARS} characters or fewer."),
            ));
        }

        errors
    }
}

impl FieldError {
    fn new(field: Field, message: impl Into<String>) -> Self {
        Self {
            field,
            message: message.into(),
        }
    }
}

/// Counts Unicode scalar values, the unit every limit in this crate uses.
pub fn char_count(text: &str) -> usize {
    text.chars().count()
}

/// Turns a title into a lowercase, dash-separated URL segment.
///
/// Letters and digits from any script are kept; everything else becomes a
/// single dash. `"Hello, Wörld!"` becomes `"hello-wörld"`.
pub fn slugify(title: &str) -> String {
    let mut slug = String::with_capacity(title.len());
    let mut pending_dash = false;

    for c in title.chars() {
        if c.is_alphanumeric() {
            if pending_dash && !slug.is_empty() {
                slug.push('-');
            }
            pending_dash = false;
            slug.extend(c.to_lowercase());
        } else {
            pending_dash = true;
        }
    }

    slug
}

#[cfg(test)]
mod tests {
    use super::*;

    fn draft(title: &str, body: &str) -> NoteDraft {
        NoteDraft {
            title: title.into(),
            body: body.into(),
        }
    }

    fn fields(errors: &[FieldError]) -> Vec<Field> {
        errors.iter().map(|e| e.field).collect()
    }

    #[test]
    fn valid_draft_has_no_errors() {
        assert!(draft("Groceries", "Milk, eggs").validate().is_empty());
    }

    #[test]
    fn body_is_optional() {
        assert!(draft("Groceries", "").validate().is_empty());
    }

    #[test]
    fn blank_title_is_rejected() {
        assert_eq!(fields(&draft("   ", "").validate()), [Field::Title]);
    }

    #[test]
    fn title_without_letters_or_digits_is_rejected() {
        assert_eq!(fields(&draft("!!!", "").validate()), [Field::Title]);
    }

    #[test]
    fn limits_count_characters_not_bytes_or_utf16_units() {
        // 80 crabs: 320 bytes in UTF-8, 160 units in UTF-16, 80 characters.
        let title = format!("a{}", "🦀".repeat(TITLE_MAX_CHARS - 1));
        assert!(draft(&title, "").validate().is_empty());

        let title = format!("a{}", "🦀".repeat(TITLE_MAX_CHARS));
        assert_eq!(fields(&draft(&title, "").validate()), [Field::Title]);
    }

    #[test]
    fn overlong_body_is_rejected() {
        let body = "x".repeat(BODY_MAX_CHARS + 1);
        assert_eq!(fields(&draft("Ok", &body).validate()), [Field::Body]);
    }

    #[test]
    fn whitespace_does_not_count_towards_limits() {
        let title = format!("  {}  ", "t".repeat(TITLE_MAX_CHARS));
        assert!(draft(&title, "").validate().is_empty());
    }

    #[test]
    fn trimmed_strips_both_fields() {
        assert_eq!(draft(" a ", "\n b \n").trimmed(), draft("a", "b"));
    }

    #[test]
    fn slugify_collapses_separators() {
        assert_eq!(slugify("  Hello,   World!  "), "hello-world");
        assert_eq!(slugify("Rust + Aurelia 2"), "rust-aurelia-2");
    }

    #[test]
    fn slugify_keeps_non_ascii_letters() {
        assert_eq!(slugify("Hello, Wörld!"), "hello-wörld");
    }

    #[test]
    fn wire_format_is_lowercase_and_camel_case() {
        let error = serde_json::to_value(FieldError::new(Field::Title, "x")).unwrap();
        assert_eq!(error["field"], "title");

        let limits = serde_json::to_value(LIMITS).unwrap();
        assert_eq!(limits["titleMaxChars"], TITLE_MAX_CHARS);
    }
}
