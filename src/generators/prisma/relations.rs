use crate::types::{Field, SchemaType};
use crate::utils::{is_prisma_reserved, sanitize_identifier, to_pascal_case};
use std::collections::HashMap;

use super::types::generate_prisma_type;

pub fn generate_field_attributes(field: &Field) -> String {
    let mut attrs = Vec::new();

    if field.metadata.is_unique {
        attrs.push("@unique".to_string());
    }

    if matches!(field.field_type, SchemaType::Uuid) && field.name.to_lowercase() == "id" {
        attrs.push("@default(uuid())".to_string());
    }

    let field_lower = field.name.to_lowercase();
    if matches!(field.field_type, SchemaType::DateTime) {
        if field_lower == "createdat" || field_lower == "created_at" {
            attrs.push("@default(now())".to_string());
        } else if field_lower == "updatedat" || field_lower == "updated_at" {
            attrs.push("@updatedAt".to_string());
        }
    }

    if attrs.is_empty() {
        String::new()
    } else {
        format!(" {}", attrs.join(" "))
    }
}

pub fn generate_field(
    field: &Field,
    indent: &str,
    type_refs: &HashMap<String, String>,
    generate_relations: bool,
) -> Option<Vec<String>> {
    let field_name = format_field_name(&field.name);
    let mut lines = Vec::new();

    match &field.field_type {
        SchemaType::Object(obj) => {
            if !generate_relations {
                return None;
            }

            if obj.fields.is_empty() {
                let optional = if field.optional { "?" } else { "" };
                lines.push(format!("{}{} Json{}", indent, field_name, optional));
            } else {
                let related_model = type_refs
                    .get(&format!("{:?}", obj))
                    .map(|s| format_model_name(s))
                    .unwrap_or_else(|| format_model_name(&field.name));
                let optional = if field.optional { "?" } else { "" };

                lines.push(format!("{}{} {}{}", indent, field_name, related_model, optional));
                lines.push(format!("{}{}Id Int{} @unique", indent, field_name, optional));
            }
        }
        SchemaType::Array(inner) => {
            if let SchemaType::Object(obj) = inner.as_ref() {
                if !generate_relations {
                    lines.push(format!("{}{} Json", indent, field_name));
                } else if obj.fields.is_empty() {
                    lines.push(format!("{}{} Json[]", indent, field_name));
                } else {
                    let related_model = type_refs
                        .get(&format!("{:?}", obj))
                        .map(|s| format_model_name(s))
                        .unwrap_or_else(|| format_model_name(&field.name));
                    lines.push(format!("{}{} {}[]", indent, field_name, related_model));
                }
            } else if let Some(prisma_type) = generate_prisma_type(&field.field_type) {
                lines.push(format!("{}{} {}", indent, field_name, prisma_type));
            }
        }
        _ => {
            if let Some(prisma_type) = generate_prisma_type(&field.field_type) {
                let optional = if field.optional { "?" } else { "" };
                let attrs = generate_field_attributes(field);
                lines.push(format!(
                    "{}{} {}{}{}",
                    indent, field_name, prisma_type, optional, attrs
                ));
            }
        }
    }

    if lines.is_empty() {
        None
    } else {
        Some(lines)
    }
}

pub fn format_field_name(name: &str) -> String {
    let sanitized = sanitize_identifier(name);
    if is_prisma_reserved(&sanitized) {
        format!("{}_", sanitized)
    } else {
        sanitized
    }
}

pub fn format_model_name(name: &str) -> String {
    let pascal = to_pascal_case(name);
    if is_prisma_reserved(&pascal) {
        format!("{}Model", pascal)
    } else {
        pascal
    }
}
