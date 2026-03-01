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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{FieldMetadata, ObjectType};

    // --- format_field_name tests ---

    #[test]
    fn test_format_field_name_normal() {
        assert_eq!(format_field_name("name"), "name");
        assert_eq!(format_field_name("user_id"), "user_id");
    }

    #[test]
    fn test_format_field_name_reserved() {
        assert_eq!(format_field_name("model"), "model_");
        assert_eq!(format_field_name("enum"), "enum_");
        assert_eq!(format_field_name("type"), "type_");
    }

    #[test]
    fn test_format_field_name_with_special_chars() {
        assert_eq!(format_field_name("my-field"), "my_field");
        assert_eq!(format_field_name("123key"), "_123key");
    }

    // --- format_model_name tests ---

    #[test]
    fn test_format_model_name_normal() {
        assert_eq!(format_model_name("user"), "User");
        assert_eq!(format_model_name("blog_post"), "BlogPost");
    }

    #[test]
    fn test_format_model_name_reserved() {
        // to_pascal_case converts to "Model"/"Enum"/"Type" which aren't in
        // the lowercase reserved list, so they pass through unchanged
        assert_eq!(format_model_name("model"), "Model");
        assert_eq!(format_model_name("enum"), "Enum");
        assert_eq!(format_model_name("type"), "Type");
    }

    // --- generate_field_attributes tests ---

    #[test]
    fn test_field_attributes_empty() {
        let field = Field::new("name", SchemaType::String);
        assert_eq!(generate_field_attributes(&field), "");
    }

    #[test]
    fn test_field_attributes_unique() {
        let mut metadata = FieldMetadata::new();
        metadata.is_unique = true;
        let field = Field::new("email", SchemaType::Email).with_metadata(metadata);
        assert_eq!(generate_field_attributes(&field), " @unique");
    }

    #[test]
    fn test_field_attributes_uuid_id() {
        let field = Field::new("id", SchemaType::Uuid);
        assert_eq!(generate_field_attributes(&field), " @default(uuid())");
    }

    #[test]
    fn test_field_attributes_uuid_non_id() {
        let field = Field::new("ref_id", SchemaType::Uuid);
        assert_eq!(generate_field_attributes(&field), "");
    }

    #[test]
    fn test_field_attributes_created_at() {
        let field = Field::new("createdAt", SchemaType::DateTime);
        assert_eq!(generate_field_attributes(&field), " @default(now())");
    }

    #[test]
    fn test_field_attributes_created_at_snake() {
        let field = Field::new("created_at", SchemaType::DateTime);
        assert_eq!(generate_field_attributes(&field), " @default(now())");
    }

    #[test]
    fn test_field_attributes_updated_at() {
        let field = Field::new("updatedAt", SchemaType::DateTime);
        assert_eq!(generate_field_attributes(&field), " @updatedAt");
    }

    #[test]
    fn test_field_attributes_regular_datetime() {
        let field = Field::new("publishedAt", SchemaType::DateTime);
        assert_eq!(generate_field_attributes(&field), "");
    }

    // --- generate_field tests ---

    #[test]
    fn test_generate_field_string() {
        let field = Field::new("name", SchemaType::String);
        let refs = HashMap::new();
        let result = generate_field(&field, "  ", &refs, true);
        assert_eq!(result, Some(vec!["  name String".to_string()]));
    }

    #[test]
    fn test_generate_field_null_returns_none() {
        let field = Field::new("nothing", SchemaType::Null);
        let refs = HashMap::new();
        let result = generate_field(&field, "  ", &refs, true);
        assert!(result.is_none());
    }

    #[test]
    fn test_generate_field_empty_object_with_relations() {
        let field = Field::new("metadata", SchemaType::Object(ObjectType::empty()));
        let refs = HashMap::new();
        let result = generate_field(&field, "  ", &refs, true);
        assert_eq!(result, Some(vec!["  metadata Json".to_string()]));
    }

    #[test]
    fn test_generate_field_object_without_relations() {
        let obj = ObjectType::new(vec![Field::new("city", SchemaType::String)]);
        let field = Field::new("address", SchemaType::Object(obj));
        let refs = HashMap::new();
        let result = generate_field(&field, "  ", &refs, false);
        assert!(result.is_none());
    }

    #[test]
    fn test_generate_field_object_array_without_relations() {
        let obj = ObjectType::new(vec![Field::new("name", SchemaType::String)]);
        let field = Field::new(
            "items",
            SchemaType::Array(Box::new(SchemaType::Object(obj))),
        );
        let refs = HashMap::new();
        let result = generate_field(&field, "  ", &refs, false);
        assert_eq!(result, Some(vec!["  items Json".to_string()]));
    }
}
