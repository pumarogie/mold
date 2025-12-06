use thiserror::Error;

/// Represents all possible JSON/Schema types
#[derive(Debug, Clone, PartialEq)]
pub enum SchemaType {
    // Primitives
    String,
    Number,  // Float (f64)
    Integer, // Whole numbers
    Boolean,
    Null,

    // Complex types
    Array(Box<SchemaType>),
    Object(ObjectType),

    // Special cases
    Optional(Box<SchemaType>), // For nullable fields
    Union(Vec<SchemaType>),    // Mixed arrays: [1, "two"] → number | string
    Any,                       // Fallback for empty arrays, unknown
}

/// Represents an object with named fields
#[derive(Debug, Clone, PartialEq)]
pub struct ObjectType {
    pub fields: Vec<Field>,
}

impl ObjectType {
    pub fn new(fields: Vec<Field>) -> Self {
        Self { fields }
    }

    pub fn empty() -> Self {
        Self { fields: vec![] }
    }
}

/// A single field within an object
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: String,
    pub field_type: SchemaType,
    pub optional: bool,
}

impl Field {
    pub fn new(name: impl Into<String>, field_type: SchemaType) -> Self {
        Self {
            name: name.into(),
            field_type,
            optional: false,
        }
    }

    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }
}

/// Extracted nested object (for non-flat mode)
#[derive(Debug, Clone)]
pub struct NestedType {
    pub name: String,
    pub object: ObjectType,
}

impl NestedType {
    pub fn new(name: impl Into<String>, object: ObjectType) -> Self {
        Self {
            name: name.into(),
            object,
        }
    }
}

/// Top-level schema container
#[derive(Debug, Clone)]
pub struct Schema {
    pub name: String,
    pub root_type: SchemaType,
    pub nested_types: Vec<NestedType>,
}

impl Schema {
    pub fn new(name: impl Into<String>, root_type: SchemaType) -> Self {
        Self {
            name: name.into(),
            root_type,
            nested_types: vec![],
        }
    }

    pub fn with_nested_types(mut self, nested_types: Vec<NestedType>) -> Self {
        self.nested_types = nested_types;
        self
    }
}

/// Custom error types for mold
#[derive(Debug, Error)]
pub enum MoldError {
    #[error("Failed to read file: {0}")]
    FileRead(#[from] std::io::Error),

    #[error("Invalid JSON: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("Root must be an object, got {0}")]
    InvalidRoot(String),

    #[error("Failed to write output: {0}")]
    WriteError(String),

    #[error("No output format specified. Use --ts, --zod, --prisma, or --all")]
    NoOutputFormat,
}
