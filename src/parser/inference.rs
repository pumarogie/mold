use crate::types::{Field, NestedType, ObjectType, SchemaType};
use crate::utils::path_to_type_name;
use serde_json::Value;
use std::collections::HashSet;

use super::patterns::{detect_string_type, is_semantic_string_type};

pub fn infer_type_flat(value: &Value) -> SchemaType {
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
        Value::String(s) => detect_string_type(s),
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

pub fn infer_type_with_extraction(
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
        Value::String(s) => detect_string_type(s),
        Value::Array(arr) => {
            if arr.is_empty() {
                SchemaType::Array(Box::new(SchemaType::Any))
            } else {
                let types: Vec<SchemaType> = arr
                    .iter()
                    .map(|val| {
                        if val.is_object() {
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
                        path.push(key.clone());
                        let nested_type = infer_type_with_extraction(val, path, nested_types);

                        if let SchemaType::Object(ref obj_type) = nested_type {
                            let type_name = path_to_type_name(path);
                            nested_types.push(NestedType::new(type_name.clone(), obj_type.clone()));
                            path.pop();
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

pub fn unify_types(types: &[SchemaType]) -> SchemaType {
    if types.is_empty() {
        return SchemaType::Any;
    }

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

    let all_string_like = unique
        .iter()
        .all(|t| matches!(t, SchemaType::String) || is_semantic_string_type(t));
    if all_string_like && unique.len() > 1 {
        return SchemaType::String;
    }

    SchemaType::Union(unique.iter().map(|t| (*t).clone()).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- unify_types tests ---

    #[test]
    fn test_unify_empty_returns_any() {
        assert_eq!(unify_types(&[]), SchemaType::Any);
    }

    #[test]
    fn test_unify_single_type() {
        assert_eq!(unify_types(&[SchemaType::String]), SchemaType::String);
        assert_eq!(unify_types(&[SchemaType::Integer]), SchemaType::Integer);
        assert_eq!(unify_types(&[SchemaType::Boolean]), SchemaType::Boolean);
    }

    #[test]
    fn test_unify_duplicate_types() {
        assert_eq!(
            unify_types(&[SchemaType::String, SchemaType::String]),
            SchemaType::String
        );
    }

    #[test]
    fn test_unify_integer_and_number_collapses_to_number() {
        assert_eq!(
            unify_types(&[SchemaType::Integer, SchemaType::Number]),
            SchemaType::Number
        );
        assert_eq!(
            unify_types(&[SchemaType::Number, SchemaType::Integer]),
            SchemaType::Number
        );
    }

    #[test]
    fn test_unify_integer_number_and_string_collapses_integer() {
        let result = unify_types(&[SchemaType::Integer, SchemaType::Number, SchemaType::String]);
        if let SchemaType::Union(types) = result {
            assert!(!types.contains(&SchemaType::Integer));
            assert!(types.contains(&SchemaType::Number));
            assert!(types.contains(&SchemaType::String));
        } else {
            panic!("Expected Union type");
        }
    }

    #[test]
    fn test_unify_string_and_semantic_collapses_to_string() {
        assert_eq!(
            unify_types(&[SchemaType::String, SchemaType::Email]),
            SchemaType::String
        );
        assert_eq!(
            unify_types(&[SchemaType::Uuid, SchemaType::String]),
            SchemaType::String
        );
        assert_eq!(
            unify_types(&[SchemaType::DateTime, SchemaType::Url, SchemaType::String]),
            SchemaType::String
        );
    }

    #[test]
    fn test_unify_mixed_types_creates_union() {
        let result = unify_types(&[SchemaType::String, SchemaType::Boolean]);
        assert!(matches!(result, SchemaType::Union(_)));
        if let SchemaType::Union(types) = result {
            assert_eq!(types.len(), 2);
        }
    }

    #[test]
    fn test_unify_three_distinct_types() {
        let result = unify_types(&[SchemaType::String, SchemaType::Boolean, SchemaType::Null]);
        if let SchemaType::Union(types) = result {
            assert_eq!(types.len(), 3);
        } else {
            panic!("Expected Union type");
        }
    }

    // --- infer_type_flat tests ---

    #[test]
    fn test_infer_flat_null() {
        let val = serde_json::Value::Null;
        assert_eq!(infer_type_flat(&val), SchemaType::Null);
    }

    #[test]
    fn test_infer_flat_boolean() {
        assert_eq!(infer_type_flat(&serde_json::json!(true)), SchemaType::Boolean);
        assert_eq!(infer_type_flat(&serde_json::json!(false)), SchemaType::Boolean);
    }

    #[test]
    fn test_infer_flat_integer() {
        assert_eq!(infer_type_flat(&serde_json::json!(42)), SchemaType::Integer);
        assert_eq!(infer_type_flat(&serde_json::json!(0)), SchemaType::Integer);
        assert_eq!(infer_type_flat(&serde_json::json!(-5)), SchemaType::Integer);
    }

    #[test]
    fn test_infer_flat_number() {
        assert_eq!(infer_type_flat(&serde_json::json!(3.14)), SchemaType::Number);
        assert_eq!(infer_type_flat(&serde_json::json!(-0.5)), SchemaType::Number);
    }

    #[test]
    fn test_infer_flat_string() {
        assert_eq!(infer_type_flat(&serde_json::json!("hello")), SchemaType::String);
    }

    #[test]
    fn test_infer_flat_semantic_string() {
        assert_eq!(
            infer_type_flat(&serde_json::json!("user@example.com")),
            SchemaType::Email
        );
        assert_eq!(
            infer_type_flat(&serde_json::json!("550e8400-e29b-41d4-a716-446655440000")),
            SchemaType::Uuid
        );
    }

    #[test]
    fn test_infer_flat_empty_array() {
        assert_eq!(
            infer_type_flat(&serde_json::json!([])),
            SchemaType::Array(Box::new(SchemaType::Any))
        );
    }

    #[test]
    fn test_infer_flat_string_array() {
        assert_eq!(
            infer_type_flat(&serde_json::json!(["a", "b", "c"])),
            SchemaType::Array(Box::new(SchemaType::String))
        );
    }

    #[test]
    fn test_infer_flat_mixed_array() {
        let result = infer_type_flat(&serde_json::json!([1, "two", true]));
        if let SchemaType::Array(inner) = result {
            assert!(matches!(*inner, SchemaType::Union(_)));
        } else {
            panic!("Expected Array type");
        }
    }

    #[test]
    fn test_infer_flat_nested_object() {
        let result = infer_type_flat(&serde_json::json!({"name": "John", "age": 30}));
        if let SchemaType::Object(obj) = result {
            assert_eq!(obj.fields.len(), 2);
        } else {
            panic!("Expected Object type");
        }
    }

    // --- infer_type_with_extraction tests ---

    #[test]
    fn test_extraction_extracts_nested_object() {
        let val = serde_json::json!({
            "name": "John",
            "address": {
                "street": "123 Main St",
                "city": "Springfield"
            }
        });
        let mut path = vec!["Root".to_string()];
        let mut nested = Vec::new();
        let result = infer_type_with_extraction(&val, &mut path, &mut nested);

        assert!(matches!(result, SchemaType::Object(_)));
        assert_eq!(nested.len(), 1);
        assert_eq!(nested[0].name, "RootAddress");
    }

    #[test]
    fn test_extraction_deeply_nested() {
        let val = serde_json::json!({
            "user": {
                "profile": {
                    "bio": "Developer"
                }
            }
        });
        let mut path = vec!["Root".to_string()];
        let mut nested = Vec::new();
        infer_type_with_extraction(&val, &mut path, &mut nested);

        assert_eq!(nested.len(), 2);
        let names: Vec<&str> = nested.iter().map(|n| n.name.as_str()).collect();
        assert!(names.contains(&"RootUserProfile"));
        assert!(names.contains(&"RootUser"));
    }

    #[test]
    fn test_extraction_skips_empty_objects() {
        let val = serde_json::json!({
            "name": "John",
            "metadata": {}
        });
        let mut path = vec!["Root".to_string()];
        let mut nested = Vec::new();
        infer_type_with_extraction(&val, &mut path, &mut nested);

        assert!(nested.is_empty());
    }

    #[test]
    fn test_extraction_preserves_path_after_processing() {
        let val = serde_json::json!({
            "address": { "city": "NY" }
        });
        let mut path = vec!["Root".to_string()];
        let mut nested = Vec::new();
        infer_type_with_extraction(&val, &mut path, &mut nested);

        // Path should be back to just ["Root"] after processing
        assert_eq!(path, vec!["Root".to_string()]);
    }
}
