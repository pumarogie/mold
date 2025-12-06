use crate::types::{MoldError, Schema, SchemaType};
use crate::utils::{get_file_stem, to_pascal_case};
use anyhow::Result;
use std::path::Path;

use super::inference::{infer_type_flat, infer_type_with_extraction};

pub fn parse_json_file(path: &Path, name: Option<&str>, flat_mode: bool) -> Result<Schema> {
    let content = std::fs::read_to_string(path)?;
    let root_name = name
        .map(to_pascal_case)
        .unwrap_or_else(|| to_pascal_case(get_file_stem(path).as_str()));

    parse_json_string(&content, &root_name, flat_mode)
}

pub fn parse_json_string(json: &str, name: &str, flat_mode: bool) -> Result<Schema> {
    let value: serde_json::Value = serde_json::from_str(json)?;
    parse_json_value(&value, name, flat_mode)
}

pub fn parse_json_value(value: &serde_json::Value, name: &str, flat_mode: bool) -> Result<Schema> {
    let mut nested_types = Vec::new();
    let mut path = vec![name.to_string()];

    let root_type = if flat_mode {
        infer_type_flat(value)
    } else {
        infer_type_with_extraction(value, &mut path, &mut nested_types)
    };

    if !matches!(root_type, SchemaType::Object(_)) {
        return Err(MoldError::InvalidRoot(format!("{:?}", value)).into());
    }

    Ok(Schema::new(name, root_type).with_nested_types(nested_types))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_object() {
        let json = r#"{"name": "John", "age": 30}"#;
        let schema = parse_json_string(json, "User", true).unwrap();

        assert_eq!(schema.name, "User");
        if let SchemaType::Object(obj) = &schema.root_type {
            assert_eq!(obj.fields.len(), 2);
        } else {
            panic!("Expected Object type");
        }
    }

    #[test]
    fn test_infer_integer_vs_number() {
        let int_json = r#"{"count": 42}"#;
        let float_json = r#"{"price": 19.99}"#;

        let int_schema = parse_json_string(int_json, "Test", true).unwrap();
        let float_schema = parse_json_string(float_json, "Test", true).unwrap();

        if let SchemaType::Object(obj) = &int_schema.root_type {
            assert_eq!(obj.fields[0].field_type, SchemaType::Integer);
        }

        if let SchemaType::Object(obj) = &float_schema.root_type {
            assert_eq!(obj.fields[0].field_type, SchemaType::Number);
        }
    }

    #[test]
    fn test_parse_array() {
        let json = r#"{"tags": ["a", "b", "c"]}"#;
        let schema = parse_json_string(json, "Test", true).unwrap();

        if let SchemaType::Object(obj) = &schema.root_type {
            if let SchemaType::Array(inner) = &obj.fields[0].field_type {
                assert_eq!(**inner, SchemaType::String);
            } else {
                panic!("Expected Array type");
            }
        }
    }

    #[test]
    fn test_parse_empty_array() {
        let json = r#"{"items": []}"#;
        let schema = parse_json_string(json, "Test", true).unwrap();

        if let SchemaType::Object(obj) = &schema.root_type {
            if let SchemaType::Array(inner) = &obj.fields[0].field_type {
                assert_eq!(**inner, SchemaType::Any);
            } else {
                panic!("Expected Array type");
            }
        }
    }

    #[test]
    fn test_parse_mixed_array() {
        let json = r#"{"mixed": [1, "two", true]}"#;
        let schema = parse_json_string(json, "Test", true).unwrap();

        if let SchemaType::Object(obj) = &schema.root_type {
            if let SchemaType::Array(inner) = &obj.fields[0].field_type {
                assert!(matches!(**inner, SchemaType::Union(_)));
            } else {
                panic!("Expected Array type");
            }
        }
    }

    #[test]
    fn test_nested_extraction() {
        let json = r#"{
            "user": {
                "profile": {
                    "name": "John"
                }
            }
        }"#;
        let schema = parse_json_string(json, "Root", false).unwrap();
        assert!(!schema.nested_types.is_empty());
    }

    #[test]
    fn test_detect_uuid() {
        let json = r#"{"id": "550e8400-e29b-41d4-a716-446655440000"}"#;
        let schema = parse_json_string(json, "Test", true).unwrap();

        if let SchemaType::Object(obj) = &schema.root_type {
            assert_eq!(obj.fields[0].field_type, SchemaType::Uuid);
        } else {
            panic!("Expected Object type");
        }
    }

    #[test]
    fn test_detect_datetime() {
        let json = r#"{"created": "2023-01-15T10:30:00Z"}"#;
        let schema = parse_json_string(json, "Test", true).unwrap();

        if let SchemaType::Object(obj) = &schema.root_type {
            assert_eq!(obj.fields[0].field_type, SchemaType::DateTime);
        } else {
            panic!("Expected Object type");
        }
    }

    #[test]
    fn test_detect_date() {
        let json = r#"{"birthday": "2023-01-15"}"#;
        let schema = parse_json_string(json, "Test", true).unwrap();

        if let SchemaType::Object(obj) = &schema.root_type {
            assert_eq!(obj.fields[0].field_type, SchemaType::Date);
        } else {
            panic!("Expected Object type");
        }
    }

    #[test]
    fn test_detect_email() {
        let json = r#"{"email": "user@example.com"}"#;
        let schema = parse_json_string(json, "Test", true).unwrap();

        if let SchemaType::Object(obj) = &schema.root_type {
            assert_eq!(obj.fields[0].field_type, SchemaType::Email);
        } else {
            panic!("Expected Object type");
        }
    }

    #[test]
    fn test_detect_url() {
        let json = r#"{"website": "https://example.com/path"}"#;
        let schema = parse_json_string(json, "Test", true).unwrap();

        if let SchemaType::Object(obj) = &schema.root_type {
            assert_eq!(obj.fields[0].field_type, SchemaType::Url);
        } else {
            panic!("Expected Object type");
        }
    }

    #[test]
    fn test_plain_string_not_semantic() {
        let json = r#"{"name": "John Doe"}"#;
        let schema = parse_json_string(json, "Test", true).unwrap();

        if let SchemaType::Object(obj) = &schema.root_type {
            assert_eq!(obj.fields[0].field_type, SchemaType::String);
        } else {
            panic!("Expected Object type");
        }
    }
}
