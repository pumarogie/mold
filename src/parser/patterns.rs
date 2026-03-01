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

#[cfg(test)]
mod tests {
    use super::*;

    // --- detect_string_type tests ---

    #[test]
    fn test_detect_uuid_valid() {
        assert_eq!(
            detect_string_type("550e8400-e29b-41d4-a716-446655440000"),
            SchemaType::Uuid
        );
        assert_eq!(
            detect_string_type("00000000-0000-0000-0000-000000000000"),
            SchemaType::Uuid
        );
        assert_eq!(
            detect_string_type("ABCDEF01-2345-6789-ABCD-EF0123456789"),
            SchemaType::Uuid
        );
    }

    #[test]
    fn test_detect_uuid_rejects_invalid() {
        assert_eq!(detect_string_type("not-a-uuid"), SchemaType::String);
        assert_eq!(detect_string_type("550e8400-e29b-41d4-a716"), SchemaType::String);
        assert_eq!(detect_string_type("550e8400e29b41d4a716446655440000"), SchemaType::String);
    }

    #[test]
    fn test_detect_datetime_valid() {
        assert_eq!(detect_string_type("2023-01-15T10:30:00Z"), SchemaType::DateTime);
        assert_eq!(detect_string_type("2023-01-15T10:30:00+05:30"), SchemaType::DateTime);
        assert_eq!(detect_string_type("2023-01-15T10:30:00.123Z"), SchemaType::DateTime);
        assert_eq!(detect_string_type("2023-01-15T10:30:00"), SchemaType::DateTime);
    }

    #[test]
    fn test_detect_date_valid() {
        assert_eq!(detect_string_type("2023-01-15"), SchemaType::Date);
        assert_eq!(detect_string_type("1999-12-31"), SchemaType::Date);
    }

    #[test]
    fn test_detect_date_rejects_invalid() {
        assert_eq!(detect_string_type("2023-1-15"), SchemaType::String);
        assert_eq!(detect_string_type("01-15-2023"), SchemaType::String);
    }

    #[test]
    fn test_detect_email_valid() {
        assert_eq!(detect_string_type("user@example.com"), SchemaType::Email);
        assert_eq!(detect_string_type("name+tag@domain.co.uk"), SchemaType::Email);
    }

    #[test]
    fn test_detect_email_rejects_invalid() {
        assert_eq!(detect_string_type("not an email"), SchemaType::String);
        assert_eq!(detect_string_type("@no-local.com"), SchemaType::String);
    }

    #[test]
    fn test_detect_url_valid() {
        assert_eq!(detect_string_type("https://example.com"), SchemaType::Url);
        assert_eq!(detect_string_type("http://example.com/path?q=1"), SchemaType::Url);
        assert_eq!(detect_string_type("https://sub.domain.com/a/b/c"), SchemaType::Url);
    }

    #[test]
    fn test_detect_url_rejects_non_http() {
        assert_eq!(detect_string_type("ftp://example.com"), SchemaType::String);
        assert_eq!(detect_string_type("example.com"), SchemaType::String);
    }

    #[test]
    fn test_detect_plain_string() {
        assert_eq!(detect_string_type("hello world"), SchemaType::String);
        assert_eq!(detect_string_type(""), SchemaType::String);
        assert_eq!(detect_string_type("John Doe"), SchemaType::String);
        assert_eq!(detect_string_type("just a regular string"), SchemaType::String);
    }

    // --- is_semantic_string_type tests ---

    #[test]
    fn test_is_semantic_true_for_semantic_types() {
        assert!(is_semantic_string_type(&SchemaType::DateTime));
        assert!(is_semantic_string_type(&SchemaType::Date));
        assert!(is_semantic_string_type(&SchemaType::Uuid));
        assert!(is_semantic_string_type(&SchemaType::Email));
        assert!(is_semantic_string_type(&SchemaType::Url));
    }

    #[test]
    fn test_is_semantic_false_for_non_semantic_types() {
        assert!(!is_semantic_string_type(&SchemaType::String));
        assert!(!is_semantic_string_type(&SchemaType::Integer));
        assert!(!is_semantic_string_type(&SchemaType::Boolean));
        assert!(!is_semantic_string_type(&SchemaType::Null));
        assert!(!is_semantic_string_type(&SchemaType::Any));
    }
}
