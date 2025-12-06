use crate::types::SchemaType;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref UUID_RE: Regex = Regex::new(
        r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$"
    ).unwrap();

    static ref DATETIME_RE: Regex = Regex::new(
        r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?(Z|[+-]\d{2}:\d{2})?$"
    ).unwrap();

    static ref DATE_RE: Regex = Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap();

    static ref EMAIL_RE: Regex = Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").unwrap();

    static ref URL_RE: Regex = Regex::new(r"^https?://[^\s]+$").unwrap();
}

pub fn detect_string_type(s: &str) -> SchemaType {
    if UUID_RE.is_match(s) {
        SchemaType::Uuid
    } else if DATETIME_RE.is_match(s) {
        SchemaType::DateTime
    } else if DATE_RE.is_match(s) {
        SchemaType::Date
    } else if EMAIL_RE.is_match(s) {
        SchemaType::Email
    } else if URL_RE.is_match(s) {
        SchemaType::Url
    } else {
        SchemaType::String
    }
}

pub fn is_semantic_string_type(t: &SchemaType) -> bool {
    matches!(
        t,
        SchemaType::DateTime
            | SchemaType::Date
            | SchemaType::Uuid
            | SchemaType::Email
            | SchemaType::Url
    )
}
