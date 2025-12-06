use crate::types::{Field, MoldError, NestedType, ObjectType, Schema, SchemaType};
use crate::utils::{path_to_type_name, to_pascal_case};
use anyhow::Result;
use serde_json::Value;
use std::collections::HashSet;
use std::path::Path;

/// Parse a JSON file into a Schema
pub fn parse_json_file(path: &Path, name: Option<&str>, flat_mode: bool) -> Result<Schema> {
    let content = std::fs::read_to_string(path)?;
    let root_name = name
        .map(|s| to_pascal_case(s))
        .unwrap_or_else(|| to_pascal_case(crate::utils::get_file_stem(path).as_str()));

    parse_json_string(&content, &root_name, flat_mode)
}

/// Parse a JSON string into a Schema
pub fn parse_json_string(json: &str, name: &str, flat_mode: bool) -> Result<Schema> {
    let value: Value = serde_json::from_str(json)?;
    parse_json_value(&value, name, flat_mode)
}

/// Parse a JSON Value into a Schema
pub fn parse_json_value(value: &Value, name: &str, flat_mode: bool) -> Result<Schema> {
    let mut nested_types = Vec::new();
    let mut path = vec![name.to_string()];

    let root_type = if flat_mode {
        infer_type_flat(value)
    } else {
        infer_type_with_extraction(value, &mut path, &mut nested_types)
    };

    // Ensure root is an object
    if !matches!(root_type, SchemaType::Object(_)) {
        return Err(MoldError::InvalidRoot(format!("{:?}", value)).into());
    }

    Ok(Schema::new(name, root_type).with_nested_types(nested_types))
}

/// Infer type from JSON value (flat mode - no extraction)
fn infer_type_flat(value: &Value) -> SchemaType {
    match value {
        Value::Null => SchemaType::Null,
        Value::Bool(_) => SchemaType::Boolean,
        Value::Number(n) => {
            if n.is_i64() || n.is_u64() {
                // Check if it's a whole number
                if let Some(f) = n.as_f64() {
                    if f.fract() == 0.0 {
                        return SchemaType::Integer;
                    }
                }
            }
            SchemaType::Number
        }
        Value::String(_) => SchemaType::String,
        Value::Array(arr) => {
            if arr.is_empty() {
                SchemaType::Array(Box::new(SchemaType::Any))
            } else {
                let types: Vec<SchemaType> = arr.iter().map(infer_type_flat).collect();
                let unified = unify_types(&types);
                SchemaType::Array(Box::new(unified))
            }
        }
        Value::Object(obj) => {
            let fields: Vec<Field> = obj
                .iter()
                .map(|(key, val)| Field::new(key.clone(), infer_type_flat(val)))
                .collect();
            SchemaType::Object(ObjectType::new(fields))
        }
    }
}

/// Infer type from JSON value with nested type extraction
fn infer_type_with_extraction(
    value: &Value,
    path: &mut Vec<String>,
    nested_types: &mut Vec<NestedType>,
) -> SchemaType {
    match value {
        Value::Null => SchemaType::Null,
        Value::Bool(_) => SchemaType::Boolean,
        Value::Number(n) => {
            if n.is_i64() || n.is_u64() {
                if let Some(f) = n.as_f64() {
                    if f.fract() == 0.0 {
                        return SchemaType::Integer;
                    }
                }
            }
            SchemaType::Number
        }
        Value::String(_) => SchemaType::String,
        Value::Array(arr) => {
            if arr.is_empty() {
                SchemaType::Array(Box::new(SchemaType::Any))
            } else {
                // For arrays, we need to handle object extraction differently
                let types: Vec<SchemaType> = arr
                    .iter()
                    .map(|val| {
                        if val.is_object() {
                            // For objects in arrays, use singular form of parent + "Item"
                            path.push("Item".to_string());
                            let t = infer_type_with_extraction(val, path, nested_types);
                            path.pop();
                            t
                        } else {
                            infer_type_with_extraction(val, path, nested_types)
                        }
                    })
                    .collect();
                let unified = unify_types(&types);
                SchemaType::Array(Box::new(unified))
            }
        }
        Value::Object(obj) => {
            let fields: Vec<Field> = obj
                .iter()
                .map(|(key, val)| {
                    let field_type = if val.is_object() && !val.as_object().unwrap().is_empty() {
                        // This is a nested object - extract it
                        path.push(key.clone());
                        let nested_type =
                            infer_type_with_extraction(val, path, nested_types);

                        // Extract to nested_types if it's an object
                        if let SchemaType::Object(ref obj_type) = nested_type {
                            let type_name = path_to_type_name(path);
                            nested_types.push(NestedType::new(type_name.clone(), obj_type.clone()));
                            path.pop();
                            // Return a reference to the extracted type
                            // We'll use Object with empty fields as a marker, and store the name
                            // Actually, let's create a special handling for this
                            return Field::new(key.clone(), SchemaType::Object(obj_type.clone()));
                        }
                        path.pop();
                        nested_type
                    } else {
                        infer_type_with_extraction(val, path, nested_types)
                    };
                    Field::new(key.clone(), field_type)
                })
                .collect();
            SchemaType::Object(ObjectType::new(fields))
        }
    }
}

/// Unify multiple types into a single type
fn unify_types(types: &[SchemaType]) -> SchemaType {
    if types.is_empty() {
        return SchemaType::Any;
    }

    // Deduplicate types
    let unique: Vec<&SchemaType> = {
        let mut seen = HashSet::new();
        types
            .iter()
            .filter(|t| {
                let key = format!("{:?}", t);
                seen.insert(key)
            })
            .collect()
    };

    if unique.len() == 1 {
        return unique[0].clone();
    }

    // If we have Integer and Number, prefer Number
    let has_integer = unique.iter().any(|t| matches!(t, SchemaType::Integer));
    let has_number = unique.iter().any(|t| matches!(t, SchemaType::Number));
    if has_integer && has_number {
        let filtered: Vec<SchemaType> = unique
            .iter()
            .filter(|t| !matches!(t, SchemaType::Integer))
            .map(|t| (*t).clone())
            .collect();
        if filtered.len() == 1 {
            return filtered[0].clone();
        }
        return SchemaType::Union(filtered);
    }

    // Multiple different types - create a union
    SchemaType::Union(unique.iter().map(|t| (*t).clone()).collect())
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

        // Should have extracted nested types
        assert!(!schema.nested_types.is_empty());
    }
}
