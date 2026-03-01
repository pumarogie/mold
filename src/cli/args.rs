use clap::{Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use std::path::PathBuf;

#[derive(Clone, Debug, ValueEnum)]
pub enum ColorMode {
    Auto,
    Always,
    Never,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Generate shell completion scripts
    Completions {
        /// Target shell
        #[arg(value_enum)]
        shell: Shell,
    },
}

#[derive(Parser, Debug)]
#[command(
    name = "mold",
    version,
    about = "Shape JSON into TypeScript, Zod, and Prisma",
    long_about = "\
mold infers types from JSON data and generates production-ready \
TypeScript interfaces, Zod validation schemas, and Prisma models.

It automatically detects UUIDs, emails, URLs, dates, and timestamps, \
extracts nested objects as separate types, and handles union types in arrays.",
    after_help = "\x1b[1mExamples:\x1b[0m
  mold data.json --ts
  mold api.json --all -o ./types
  mold user.json --zod --strict --name User
  cat data.json | mold --ts --name MyType
  mold a.json b.json --ts -o ./generated
  mold data.json --ts --export --readonly"
)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// JSON files to process (use - for stdin, omit to read from pipe)
    #[arg(value_name = "FILE")]
    pub files: Vec<PathBuf>,

    // -- Output Formats --
    /// Generate TypeScript interfaces
    #[arg(short = 't', long, help_heading = "Output Formats")]
    pub ts: bool,

    /// Generate Zod schema
    #[arg(short = 'z', long, help_heading = "Output Formats")]
    pub zod: bool,

    /// Generate Prisma model
    #[arg(short = 'p', long, help_heading = "Output Formats")]
    pub prisma: bool,

    /// Generate all formats (TypeScript + Zod + Prisma)
    #[arg(short = 'a', long, help_heading = "Output Formats")]
    pub all: bool,

    // -- Output --
    /// Output directory (default: stdout)
    #[arg(short = 'o', long, value_name = "DIR", help_heading = "Output")]
    pub output: Option<PathBuf>,

    /// Root type name (default: inferred from filename)
    #[arg(short = 'n', long, value_name = "NAME", help_heading = "Output")]
    pub name: Option<String>,

    /// Keep nested objects inline (no extraction)
    #[arg(long, help_heading = "Output")]
    pub flat: bool,

    /// Control color output
    #[arg(long, value_enum, default_value_t = ColorMode::Auto, help_heading = "Output")]
    pub color: ColorMode,

    /// Suppress non-essential output (only emit generated code)
    #[arg(short = 'q', long, help_heading = "Output")]
    pub quiet: bool,

    /// Show detailed inference information
    #[arg(short = 'v', long, help_heading = "Output")]
    pub verbose: bool,

    /// Watch file for changes and regenerate
    #[arg(short = 'w', long, help_heading = "Output")]
    pub watch: bool,

    // -- TypeScript Options --
    /// Add 'export' keyword to TypeScript interfaces
    #[arg(long = "export", help_heading = "TypeScript Options")]
    pub ts_export: bool,

    /// Add 'readonly' modifier to TypeScript fields
    #[arg(long = "readonly", help_heading = "TypeScript Options")]
    pub ts_readonly: bool,

    // -- Zod Options --
    /// Use .strict() for Zod object schemas
    #[arg(long = "strict", help_heading = "Zod Options")]
    pub zod_strict: bool,
}
