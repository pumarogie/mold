use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;
use mold_cli::cli::Args;
use mold_cli::generators::{Generator, GeneratorConfig, PrismaGenerator, TypeScriptGenerator, ZodGenerator};
use mold_cli::parser::parse_json_file;
use mold_cli::types::MoldError;
use mold_cli::utils::write_file;

fn main() {
    if let Err(e) = run() {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = Args::parse();

    let (ts, zod, prisma) = if args.all {
        (true, true, true)
    } else {
        (args.ts, args.zod, args.prisma)
    };

    if !ts && !zod && !prisma {
        return Err(MoldError::NoOutputFormat.into());
    }

    if !args.file.exists() {
        return Err(anyhow::anyhow!(
            "Cannot find file '{}'",
            args.file.display()
        ));
    }

    let schema = parse_json_file(&args.file, args.name.as_deref(), args.flat)
        .with_context(|| format!("Failed to parse '{}'", args.file.display()))?;

    let config = GeneratorConfig {
        flat_mode: args.flat,
        indent: "  ".to_string(),
        ts_export_interfaces: args.ts_export,
        ts_readonly_fields: args.ts_readonly,
        zod_strict_objects: args.zod_strict,
        prisma_generate_relations: true,
    };

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

    if let Some(output_dir) = args.output {
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
