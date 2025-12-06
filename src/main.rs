use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;
use mold_cli::generators::{Generator, GeneratorConfig, PrismaGenerator, TypeScriptGenerator, ZodGenerator};
use mold_cli::parser::parse_json_file;
use mold_cli::types::MoldError;
use mold_cli::utils::write_file;
use std::path::PathBuf;

/// JSON to TypeScript/Zod/Prisma generator
#[derive(Parser, Debug)]
#[command(name = "mold")]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to JSON file
    #[arg(value_name = "FILE")]
    file: PathBuf,

    /// Generate TypeScript interfaces
    #[arg(short = 't', long)]
    ts: bool,

    /// Generate Zod schema
    #[arg(short = 'z', long)]
    zod: bool,

    /// Generate Prisma model
    #[arg(short = 'p', long)]
    prisma: bool,

    /// Generate all formats
    #[arg(short = 'a', long)]
    all: bool,

    /// Output directory (default: stdout)
    #[arg(short = 'o', long, value_name = "DIR")]
    output: Option<PathBuf>,

    /// Root type name (default: inferred from filename)
    #[arg(short = 'n', long, value_name = "NAME")]
    name: Option<String>,

    /// Keep nested objects inline (no extraction)
    #[arg(long)]
    flat: bool,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = Args::parse();

    // Validate that at least one output format is selected
    let (ts, zod, prisma) = if args.all {
        (true, true, true)
    } else {
        (args.ts, args.zod, args.prisma)
    };

    if !ts && !zod && !prisma {
        return Err(MoldError::NoOutputFormat.into());
    }

    // Validate input file exists
    if !args.file.exists() {
        return Err(anyhow::anyhow!(
            "Cannot find file '{}'",
            args.file.display()
        ));
    }

    // Parse JSON file
    let schema = parse_json_file(&args.file, args.name.as_deref(), args.flat)
        .with_context(|| format!("Failed to parse '{}'", args.file.display()))?;

    let config = GeneratorConfig {
        flat_mode: args.flat,
        indent: "  ".to_string(),
    };

    // Generate outputs
    let mut outputs: Vec<(&str, String, &str)> = Vec::new();

    if ts {
        let generator = TypeScriptGenerator::new();
        let output = generator.generate(&schema, &config)?;
        outputs.push(("TypeScript", output, generator.file_extension()));
    }

    if zod {
        let generator = ZodGenerator::new();
        let output = generator.generate(&schema, &config)?;
        outputs.push(("Zod", output, generator.file_extension()));
    }

    if prisma {
        let generator = PrismaGenerator::new();
        let output = generator.generate(&schema, &config)?;
        outputs.push(("Prisma", output, generator.file_extension()));
    }

    // Output results
    if let Some(output_dir) = args.output {
        // Write to files
        std::fs::create_dir_all(&output_dir)
            .with_context(|| format!("Failed to create directory '{}'", output_dir.display()))?;

        let base_name = args
            .file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("schema");

        for (name, content, ext) in outputs {
            let file_path = output_dir.join(format!("{}.{}", base_name, ext));
            write_file(&file_path, &content)?;
            println!(
                "{} {} → {}",
                "✓".green().bold(),
                name,
                file_path.display()
            );
        }
    } else {
        // Output to stdout
        for (i, (name, content, _)) in outputs.iter().enumerate() {
            if i > 0 {
                println!("\n{}", "─".repeat(60).dimmed());
            }
            println!("{}", format!("// {}", name).dimmed());
            println!("{}", content);
        }
    }

    Ok(())
}
