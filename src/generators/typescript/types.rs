use crate::types::{ObjectType, SchemaType};
use crate::utils::sanitize_identifier;
use std::collections::HashMap;

pub fn generate_type(
    schema_type: &SchemaType,
    indent: &str,
    type_refs: &HashMap<String, String>,
) -> String {
    match schema_type {
        SchemaType::String => "string".to_string(),
        SchemaType::Number | SchemaType::Integer => "number".to_string(),
        SchemaType::Boolean => "boolean".to_string(),
        SchemaType::Null => "null".to_string(),
        SchemaType::Any => "unknown".to_string(),
        SchemaType::DateTime => "string".to_string(),
        SchemaType::Date => "string".to_string(),
        SchemaType::Uuid => "string".to_string(),
        SchemaType::Email => "string".to_string(),
        SchemaType::Url => "string".to_string(),
        SchemaType::Enum(values) => {
            values
                .iter()
                .map(|v| format!("\"{}\"", v))
                .collect::<Vec<_>>()
                .join(" | ")
        }
        SchemaType::Array(inner) => {
            let inner_type = generate_type(inner, indent, type_refs);
            if matches!(**inner, SchemaType::Union(_) | SchemaType::Enum(_)) {
                format!("({})[]", inner_type)
            } else {
                format!("{}[]", inner_type)
            }
        }
        SchemaType::Optional(inner) => {
            let inner_type = generate_type(inner, indent, type_refs);
            format!("{} | undefined", inner_type)
        }
        SchemaType::Union(types) => {
            let mut type_strings: Vec<String> = types
                .iter()
                .map(|t| generate_type(t, indent, type_refs))
                .collect();
            type_strings.sort();
            type_strings.dedup();
            if type_strings.len() == 1 {
                type_strings[0].clone()
            } else {
                type_strings.join(" | ")
            }
        }
        SchemaType::Object(obj) => {
            let obj_key = format!("{:?}", obj);
            if let Some(type_name) = type_refs.get(&obj_key) {
                type_name.clone()
            } else {
                generate_inline_object(obj, indent, type_refs)
            }
        }
    }
}

pub fn generate_inline_object(
    obj: &ObjectType,
    indent: &str,
    type_refs: &HashMap<String, String>,
) -> String {
    if obj.fields.is_empty() {
        return "Record<string, unknown>".to_string();
    }

    let mut lines = vec!["{".to_string()];
    for field in &obj.fields {
        let field_name = format_field_name(&field.name);
        let field_type = generate_type(&field.field_type, &format!("{}  ", indent), type_refs);
        let optional = if field.optional { "?" } else { "" };
        lines.push(format!(
            "{}  {}{}: {};",
            indent, field_name, optional, field_type
        ));
    }
    lines.push(format!("{}}}", indent));
    lines.join("\n")
}

pub fn format_field_name(name: &str) -> String {
    let sanitized = sanitize_identifier(name);
    if name != sanitized || crate::utils::is_ts_reserved(name) || name.contains('-') || name.contains(' ') {
        format!("\"{}\"", name)
    } else {
        name.to_string()
    }
}
