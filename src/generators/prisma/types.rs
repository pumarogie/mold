use crate::types::SchemaType;

pub fn generate_prisma_type(schema_type: &SchemaType) -> Option<String> {
    match schema_type {
        SchemaType::String => Some("String".to_string()),
        SchemaType::Number => Some("Float".to_string()),
        SchemaType::Integer => Some("Int".to_string()),
        SchemaType::Boolean => Some("Boolean".to_string()),
        SchemaType::DateTime => Some("DateTime".to_string()),
        SchemaType::Date => Some("DateTime".to_string()),
        SchemaType::Uuid => Some("String".to_string()),
        SchemaType::Email => Some("String".to_string()),
        SchemaType::Url => Some("String".to_string()),
        SchemaType::Enum(_) => Some("String".to_string()),
        SchemaType::Array(inner) => match inner.as_ref() {
            SchemaType::String => Some("String[]".to_string()),
            SchemaType::Integer => Some("Int[]".to_string()),
            SchemaType::Number => Some("Float[]".to_string()),
            SchemaType::Boolean => Some("Boolean[]".to_string()),
            SchemaType::DateTime | SchemaType::Date => Some("DateTime[]".to_string()),
            SchemaType::Uuid | SchemaType::Email | SchemaType::Url => Some("String[]".to_string()),
            _ => Some("Json".to_string()),
        },
        SchemaType::Null => None,
        SchemaType::Optional(inner) => generate_prisma_type(inner).map(|t| format!("{}?", t)),
        SchemaType::Any | SchemaType::Union(_) => Some("Json".to_string()),
        SchemaType::Object(_) => None,
    }
}
