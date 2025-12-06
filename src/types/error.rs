use thiserror::Error;

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
