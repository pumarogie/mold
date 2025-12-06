use crate::types::{ObjectType, SchemaType};
use crate::utils::sanitize_identifier;
use std::collections::HashMap;

pub fn generate_type(
    schema_type: &SchemaType,
    indent: &str,
    type_refs: &HashMap<String, String>,
) -> String {
    match schema_type {
        SchemaType::String => "z.string()".to_string(),
        SchemaType::Number => "z.number()".to_string(),
        SchemaType::Integer => "z.number().int()".to_string(),
        SchemaType::Boolean => "z.boolean()".to_string(),
        SchemaType::Null => "z.null()".to_string(),
        SchemaType::Any => "z.unknown()".to_string(),
        SchemaType::DateTime => "z.string().datetime()".to_string(),
        SchemaType::Date => "z.string().date()".to_string(),
        SchemaType::Uuid => "z.string().uuid()".to_string(),
        SchemaType::Email => "z.string().email()".to_string(),
        SchemaType::Url => "z.string().url()".to_string(),
        SchemaType::Enum(values) => {
            if values.len() == 1 {
                format!("z.literal(\"{}\")", values[0])
            } else {
                let literals: Vec<String> = values
                    .iter()
                    .map(|v| format!("z.literal(\"{}\")", v))
                    .collect();
                format!("z.union([{}])", literals.join(", "))
            }
        }
        SchemaType::Array(inner) => {
            let inner_type = generate_type(inner, indent, type_refs);
            format!("z.array({})", inner_type)
        }
        SchemaType::Optional(inner) => {
            let inner_type = generate_type(inner, indent, type_refs);
            format!("{}.optional()", inner_type)
        }
        SchemaType::Union(types) => {
            if types.len() == 1 {
                return generate_type(&types[0], indent, type_refs);
            }
            let type_strings: Vec<String> = types
                .iter()
                .map(|t| generate_type(t, indent, type_refs))
                .collect();
            format!("z.union([{}])", type_strings.join(", "))
        }
        SchemaType::Object(obj) => {
            let obj_key = format!("{:?}", obj);
            if let Some(type_name) = type_refs.get(&obj_key) {
                format!("{}Schema", type_name)
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
        return "z.record(z.unknown())".to_string();
    }

    let inner_indent = format!("{}  ", indent);
    let mut lines = vec!["z.object({".to_string()];

    for field in &obj.fields {
        let field_name = format_field_name(&field.name);
        let mut field_type = generate_type(&field.field_type, &inner_indent, type_refs);
        if field.optional {
            field_type = format!("{}.optional()", field_type);
        }
        lines.push(format!("{}{}: {},", inner_indent, field_name, field_type));
    }

    lines.push(format!("{}}})", indent));
    lines.join("\n")
}

pub fn format_field_name(name: &str) -> String {
    let sanitized = sanitize_identifier(name);
    if name != sanitized || name.contains('-') || name.contains(' ') {
        format!("\"{}\"", name)
    } else {
        name.to_string()
    }
}
