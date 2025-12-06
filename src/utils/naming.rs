use convert_case::{Case, Casing};

/// Convert a string to PascalCase (e.g., "user_name" → "UserName")
pub fn to_pascal_case(s: &str) -> String {
    s.to_case(Case::Pascal)
}

/// Convert a string to camelCase (e.g., "user_name" → "userName")
pub fn to_camel_case(s: &str) -> String {
    s.to_case(Case::Camel)
}

/// Convert a string to snake_case (e.g., "UserName" → "user_name")
pub fn to_snake_case(s: &str) -> String {
    s.to_case(Case::Snake)
}

/// Sanitize an identifier to be valid in most languages
/// - Prefix with underscore if starts with a digit
/// - Replace invalid characters with underscores
pub fn sanitize_identifier(s: &str) -> String {
    if s.is_empty() {
        return "_empty".to_string();
    }

    let mut result = String::new();
    let mut chars = s.chars().peekable();

    // If starts with digit, prefix with underscore
    if let Some(first) = chars.peek() {
        if first.is_ascii_digit() {
            result.push('_');
        }
    }

    for c in chars {
        if c.is_alphanumeric() || c == '_' {
            result.push(c);
        } else {
            result.push('_');
        }
    }

    result
}

/// Check if a string is a reserved word in TypeScript/JavaScript
pub fn is_ts_reserved(s: &str) -> bool {
    matches!(
        s,
        "break"
            | "case"
            | "catch"
            | "class"
            | "const"
            | "continue"
            | "debugger"
            | "default"
            | "delete"
            | "do"
            | "else"
            | "enum"
            | "export"
            | "extends"
            | "false"
            | "finally"
            | "for"
            | "function"
            | "if"
            | "import"
            | "in"
            | "instanceof"
            | "new"
            | "null"
            | "return"
            | "super"
            | "switch"
            | "this"
            | "throw"
            | "true"
            | "try"
            | "typeof"
            | "var"
            | "void"
            | "while"
            | "with"
            | "yield"
            | "let"
            | "static"
            | "implements"
            | "interface"
            | "package"
            | "private"
            | "protected"
            | "public"
            | "type"
    )
}

/// Check if a string is a reserved word in Prisma
pub fn is_prisma_reserved(s: &str) -> bool {
    matches!(
        s,
        "model" | "enum" | "type" | "datasource" | "generator" | "true" | "false" | "null"
    )
}

/// Generate a type name from a path (e.g., ["user", "address"] → "UserAddress")
pub fn path_to_type_name(path: &[String]) -> String {
    path.iter()
        .map(|s| to_pascal_case(s))
        .collect::<Vec<_>>()
        .join("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("user_name"), "UserName");
        assert_eq!(to_pascal_case("hello_world"), "HelloWorld");
        assert_eq!(to_pascal_case("id"), "Id");
    }

    #[test]
    fn test_to_camel_case() {
        assert_eq!(to_camel_case("user_name"), "userName");
        assert_eq!(to_camel_case("HelloWorld"), "helloWorld");
    }

    #[test]
    fn test_sanitize_identifier() {
        assert_eq!(sanitize_identifier("123key"), "_123key");
        assert_eq!(sanitize_identifier("my-key"), "my_key");
        assert_eq!(sanitize_identifier("valid_name"), "valid_name");
        assert_eq!(sanitize_identifier(""), "_empty");
    }

    #[test]
    fn test_path_to_type_name() {
        assert_eq!(
            path_to_type_name(&["user".to_string(), "address".to_string()]),
            "UserAddress"
        );
        assert_eq!(
            path_to_type_name(&["profile".to_string(), "contact".to_string()]),
            "ProfileContact"
        );
    }
}
