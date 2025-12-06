use super::field::Field;
use super::schema::SchemaType;

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
