use super::object::ObjectType;

#[derive(Debug, Clone, PartialEq)]
pub enum SchemaType {
    String,
    Number,
    Integer,
    Boolean,
    Null,
    DateTime,
    Date,
    Uuid,
    Email,
    Url,
    Enum(Vec<String>),
    Array(Box<SchemaType>),
    Object(ObjectType),
    Optional(Box<SchemaType>),
    Union(Vec<SchemaType>),
    Any,
}
