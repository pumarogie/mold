use mold_cli::generators::{Generator, GeneratorConfig, PrismaGenerator, TypeScriptGenerator, ZodGenerator};
use mold_cli::parser::{parse_json_string, parse_json_value};
use mold_cli::types::SchemaType;

// =============================================================================
// End-to-end: simple.json
// =============================================================================

const SIMPLE_JSON: &str = r#"{
    "id": 1,
    "name": "John Doe",
    "email": "john@example.com",
    "active": true
}"#;

#[test]
fn test_simple_json_to_typescript() {
    let schema = parse_json_string(SIMPLE_JSON, "Simple", false).unwrap();
    let gen = TypeScriptGenerator::new();
    let config = GeneratorConfig::default();
    let output = gen.generate(&schema, &config).unwrap();

    assert!(output.contains("interface Simple {"));
    assert!(output.contains("id: number"));
    assert!(output.contains("name: string"));
    assert!(output.contains("email: string"));
    assert!(output.contains("active: boolean"));
}

#[test]
fn test_simple_json_to_zod() {
    let schema = parse_json_string(SIMPLE_JSON, "Simple", false).unwrap();
    let gen = ZodGenerator::new();
    let config = GeneratorConfig::default();
    let output = gen.generate(&schema, &config).unwrap();

    assert!(output.contains("const SimpleSchema = z.object({"));
    assert!(output.contains("z.number().int()"));
    assert!(output.contains("z.string()"));
    assert!(output.contains("z.boolean()"));
    assert!(output.contains("type Simple = z.infer<typeof SimpleSchema>"));
}

#[test]
fn test_simple_json_to_prisma() {
    let schema = parse_json_string(SIMPLE_JSON, "Simple", false).unwrap();
    let gen = PrismaGenerator::new();
    let config = GeneratorConfig::default();
    let output = gen.generate(&schema, &config).unwrap();

    assert!(output.contains("model Simple {"));
    assert!(output.contains("id Int @id @default(autoincrement())"));
    assert!(output.contains("name String"));
    assert!(output.contains("active Boolean"));
}

// =============================================================================
// End-to-end: nested.json
// =============================================================================

const NESTED_JSON: &str = r#"{
    "id": 1,
    "user": {
        "name": "Jane Smith",
        "profile": {
            "bio": "Software developer",
            "avatar": "https://example.com/avatar.png",
            "social": {
                "twitter": "@janesmith",
                "github": "janesmith"
            }
        }
    },
    "createdAt": "2024-01-15"
}"#;

#[test]
fn test_nested_json_extraction_mode() {
    let schema = parse_json_string(NESTED_JSON, "Root", false).unwrap();

    // Should extract nested types
    assert!(!schema.nested_types.is_empty());

    let gen = TypeScriptGenerator::new();
    let config = GeneratorConfig::default();
    let output = gen.generate(&schema, &config).unwrap();

    // Should have multiple interfaces
    assert!(output.contains("interface Root {"));
    // Nested types should be extracted as separate interfaces
    let interface_count = output.matches("interface ").count();
    assert!(interface_count > 1, "Expected multiple interfaces, got {}", interface_count);
}

#[test]
fn test_nested_json_flat_mode() {
    let schema = parse_json_string(NESTED_JSON, "Root", true).unwrap();

    // Flat mode should not extract nested types
    assert!(schema.nested_types.is_empty());

    let gen = TypeScriptGenerator::new();
    let mut config = GeneratorConfig::default();
    config.flat_mode = true;
    let output = gen.generate(&schema, &config).unwrap();

    // Should only have one interface
    let interface_count = output.matches("interface ").count();
    assert_eq!(interface_count, 1, "Flat mode should produce one interface");
}

// =============================================================================
// End-to-end: arrays.json
// =============================================================================

const ARRAYS_JSON: &str = r#"{
    "tags": ["typescript", "rust", "cli"],
    "scores": [98, 87, 92, 100],
    "prices": [19.99, 29.99, 9.99],
    "flags": [true, false, true],
    "mixed": [1, "two", true, null],
    "empty": []
}"#;

#[test]
fn test_arrays_json_typescript() {
    let schema = parse_json_string(ARRAYS_JSON, "ArrayTest", true).unwrap();
    let gen = TypeScriptGenerator::new();
    let config = GeneratorConfig::default();
    let output = gen.generate(&schema, &config).unwrap();

    assert!(output.contains("tags: string[]"));
    assert!(output.contains("scores: number[]"));
    assert!(output.contains("prices: number[]"));
    assert!(output.contains("flags: boolean[]"));
    assert!(output.contains("empty: unknown[]"));
    // mixed should have a union type
    assert!(output.contains("mixed:"));
}

#[test]
fn test_arrays_json_zod() {
    let schema = parse_json_string(ARRAYS_JSON, "ArrayTest", true).unwrap();
    let gen = ZodGenerator::new();
    let config = GeneratorConfig::default();
    let output = gen.generate(&schema, &config).unwrap();

    assert!(output.contains("z.array(z.string())"));
    assert!(output.contains("z.array(z.number().int())"));
    assert!(output.contains("z.array(z.number())"));
    assert!(output.contains("z.array(z.boolean())"));
    assert!(output.contains("z.array(z.unknown())"));
}

// =============================================================================
// End-to-end: blog-post.json
// =============================================================================

const BLOG_POST_JSON: &str = r#"{
    "id": 42,
    "title": "Getting Started with Rust",
    "slug": "getting-started-with-rust",
    "content": "Rust is a systems programming language...",
    "published": true,
    "views": 1523,
    "rating": 4.8,
    "author": {
        "id": 1,
        "name": "Alice Johnson",
        "email": "alice@blog.com",
        "role": "admin"
    },
    "tags": ["rust", "programming", "tutorial"],
    "comments": [
        {
            "id": 1,
            "text": "Great article!",
            "likes": 15
        },
        {
            "id": 2,
            "text": "Very helpful, thanks!",
            "likes": 8
        }
    ],
    "metadata": {
        "readTime": 5,
        "category": "Technology",
        "featured": true
    }
}"#;

#[test]
fn test_blog_post_extraction_creates_nested_types() {
    let schema = parse_json_string(BLOG_POST_JSON, "BlogPost", false).unwrap();

    // Should extract author, comments items, and metadata as nested types
    assert!(
        schema.nested_types.len() >= 2,
        "Expected at least 2 nested types, got {}",
        schema.nested_types.len()
    );
}

#[test]
fn test_blog_post_typescript() {
    let schema = parse_json_string(BLOG_POST_JSON, "BlogPost", false).unwrap();
    let gen = TypeScriptGenerator::new();
    let config = GeneratorConfig::default();
    let output = gen.generate(&schema, &config).unwrap();

    assert!(output.contains("interface BlogPost {"));
    assert!(output.contains("rating: number"));
    assert!(output.contains("views: number"));
    assert!(output.contains("published: boolean"));
    assert!(output.contains("tags: string[]"));
}

#[test]
fn test_blog_post_all_generators_succeed() {
    let schema = parse_json_string(BLOG_POST_JSON, "BlogPost", false).unwrap();
    let config = GeneratorConfig::default();

    let ts = TypeScriptGenerator::new().generate(&schema, &config);
    let zod = ZodGenerator::new().generate(&schema, &config);
    let prisma = PrismaGenerator::new().generate(&schema, &config);

    assert!(ts.is_ok(), "TypeScript generation failed");
    assert!(zod.is_ok(), "Zod generation failed");
    assert!(prisma.is_ok(), "Prisma generation failed");
}

// =============================================================================
// Semantic type detection end-to-end
// =============================================================================

#[test]
fn test_semantic_types_end_to_end() {
    let json = r#"{
        "userId": "550e8400-e29b-41d4-a716-446655440000",
        "email": "user@example.com",
        "website": "https://example.com",
        "createdAt": "2023-01-15T10:30:00Z",
        "birthday": "1990-05-20",
        "name": "John Doe"
    }"#;

    let schema = parse_json_string(json, "User", true).unwrap();

    if let SchemaType::Object(obj) = &schema.root_type {
        let field_types: Vec<(&str, &SchemaType)> = obj
            .fields
            .iter()
            .map(|f| (f.name.as_str(), &f.field_type))
            .collect();

        for (name, ftype) in &field_types {
            match *name {
                "userId" => assert_eq!(*ftype, &SchemaType::Uuid, "userId should be Uuid"),
                "email" => assert_eq!(*ftype, &SchemaType::Email, "email should be Email"),
                "website" => assert_eq!(*ftype, &SchemaType::Url, "website should be Url"),
                "createdAt" => assert_eq!(*ftype, &SchemaType::DateTime, "createdAt should be DateTime"),
                "birthday" => assert_eq!(*ftype, &SchemaType::Date, "birthday should be Date"),
                "name" => assert_eq!(*ftype, &SchemaType::String, "name should be String"),
                _ => {}
            }
        }
    } else {
        panic!("Expected Object type");
    }
}

// =============================================================================
// Error handling
// =============================================================================

#[test]
fn test_invalid_json_returns_error() {
    let result = parse_json_string("{ invalid json }", "Test", false);
    assert!(result.is_err());
}

#[test]
fn test_non_object_root_returns_error() {
    let result = parse_json_string(r#""just a string""#, "Test", false);
    assert!(result.is_err());

    let result = parse_json_string("[1, 2, 3]", "Test", false);
    assert!(result.is_err());

    let result = parse_json_string("42", "Test", false);
    assert!(result.is_err());

    let result = parse_json_string("true", "Test", false);
    assert!(result.is_err());

    let result = parse_json_string("null", "Test", false);
    assert!(result.is_err());
}

#[test]
fn test_empty_object_is_valid() {
    let result = parse_json_string("{}", "Empty", false);
    assert!(result.is_ok());
    let schema = result.unwrap();
    if let SchemaType::Object(obj) = &schema.root_type {
        assert!(obj.fields.is_empty());
    }
}

#[test]
fn test_parse_json_value_directly() {
    let value: serde_json::Value = serde_json::json!({"key": "value"});
    let result = parse_json_value(&value, "Direct", false);
    assert!(result.is_ok());
}

// =============================================================================
// Config combinations
// =============================================================================

#[test]
fn test_typescript_export_and_readonly_combined() {
    let schema = parse_json_string(SIMPLE_JSON, "User", false).unwrap();
    let gen = TypeScriptGenerator::new();
    let mut config = GeneratorConfig::default();
    config.ts_export_interfaces = true;
    config.ts_readonly_fields = true;

    let output = gen.generate(&schema, &config).unwrap();

    assert!(output.contains("export interface User {"));
    assert!(output.contains("readonly "));
}

#[test]
fn test_zod_strict_with_nested_types() {
    let schema = parse_json_string(BLOG_POST_JSON, "BlogPost", false).unwrap();
    let gen = ZodGenerator::new();
    let mut config = GeneratorConfig::default();
    config.zod_strict_objects = true;

    let output = gen.generate(&schema, &config).unwrap();

    // All objects should have .strict()
    assert!(output.contains(".strict()"));
}

// =============================================================================
// File I/O integration
// =============================================================================

#[test]
fn test_write_and_read_generated_output() {
    let schema = parse_json_string(SIMPLE_JSON, "User", false).unwrap();
    let gen = TypeScriptGenerator::new();
    let config = GeneratorConfig::default();
    let output = gen.generate(&schema, &config).unwrap();

    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("user.ts");

    mold_cli::utils::write_file(&file_path, &output).unwrap();

    let content = mold_cli::utils::read_file(&file_path).unwrap();
    assert_eq!(content, output);
}

#[test]
fn test_write_creates_nested_directories() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("a").join("b").join("c").join("output.ts");

    mold_cli::utils::write_file(&file_path, "test content").unwrap();

    let content = mold_cli::utils::read_file(&file_path).unwrap();
    assert_eq!(content, "test content");
}

#[test]
fn test_read_nonexistent_file_returns_error() {
    let result = mold_cli::utils::read_file(std::path::Path::new("/nonexistent/file.json"));
    assert!(result.is_err());
}
