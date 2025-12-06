use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "mold")]
#[command(author, version, about = "JSON to TypeScript/Zod/Prisma generator", long_about = None)]
pub struct Args {
    #[arg(value_name = "FILE")]
    pub file: PathBuf,

    #[arg(short = 't', long, help = "Generate TypeScript interfaces")]
    pub ts: bool,

    #[arg(short = 'z', long, help = "Generate Zod schema")]
    pub zod: bool,

    #[arg(short = 'p', long, help = "Generate Prisma model")]
    pub prisma: bool,

    #[arg(short = 'a', long, help = "Generate all formats")]
    pub all: bool,

    #[arg(short = 'o', long, value_name = "DIR", help = "Output directory (default: stdout)")]
    pub output: Option<PathBuf>,

    #[arg(short = 'n', long, value_name = "NAME", help = "Root type name (default: inferred from filename)")]
    pub name: Option<String>,

    #[arg(long, help = "Keep nested objects inline (no extraction)")]
    pub flat: bool,

    #[arg(long = "export", help = "Add 'export' keyword to TypeScript interfaces")]
    pub ts_export: bool,

    #[arg(long = "readonly", help = "Add 'readonly' modifier to TypeScript fields")]
    pub ts_readonly: bool,

    #[arg(long = "strict", help = "Use .strict() for Zod object schemas")]
    pub zod_strict: bool,
}
