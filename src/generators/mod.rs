mod prisma;
mod typescript;
mod zod;

pub use prisma::PrismaGenerator;
pub use typescript::TypeScriptGenerator;
pub use zod::ZodGenerator;

use crate::types::Schema;
use anyhow::Result;

/// Configuration for generators
#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    pub flat_mode: bool,
    pub indent: String,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            flat_mode: false,
            indent: "  ".to_string(),
        }
    }
}

/// Trait for all code generators
pub trait Generator {
    fn generate(&self, schema: &Schema, config: &GeneratorConfig) -> Result<String>;
    fn file_extension(&self) -> &'static str;
}
