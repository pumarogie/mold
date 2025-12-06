use super::schema::SchemaType;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct FieldMetadata {
    pub description: Option<String>,
    pub default_value: Option<String>,
    pub is_unique: bool,
    pub is_readonly: bool,
}

impl FieldMetadata {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: String,
    pub field_type: SchemaType,
    pub optional: bool,
    pub metadata: FieldMetadata,
}

impl Field {
    pub fn new(name: impl Into<String>, field_type: SchemaType) -> Self {
        Self {
            name: name.into(),
            field_type,
            optional: false,
            metadata: FieldMetadata::default(),
        }
    }

    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }

    pub fn with_metadata(mut self, metadata: FieldMetadata) -> Self {
        self.metadata = metadata;
        self
    }
}
