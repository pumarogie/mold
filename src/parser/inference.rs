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
