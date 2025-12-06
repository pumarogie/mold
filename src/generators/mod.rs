mod prisma;
mod typescript;
mod zod;

pub use prisma::PrismaGenerator;
pub use typescript::TypeScriptGenerator;
pub use zod::ZodGenerator;

use crate::types::Schema;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    pub flat_mode: bool,
    pub indent: String,
    pub ts_export_interfaces: bool,
    pub ts_readonly_fields: bool,
    pub zod_strict_objects: bool,
    pub prisma_generate_relations: bool,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            flat_mode: false,
            indent: "  ".to_string(),
            ts_export_interfaces: false,
            ts_readonly_fields: false,
            zod_strict_objects: false,
            prisma_generate_relations: true,
        }
    }
}

pub trait Generator {
    fn generate(&self, schema: &Schema, config: &GeneratorConfig) -> Result<String>;
    fn file_extension(&self) -> &'static str;
}
