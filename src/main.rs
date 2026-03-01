use anyhow::{Context, Result};
use clap::{CommandFactory, Parser};
use colored::Colorize;
use std::io::{self, IsTerminal, Read, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

use mold_cli::cli::{Args, ColorMode, Command};
use mold_cli::generators::{Generator, GeneratorConfig, PrismaGenerator, TypeScriptGenerator, ZodGenerator};
use mold_cli::parser::parse_json_string;
use mold_cli::types::{MoldError, Schema, SchemaType};
use mold_cli::utils::{get_file_stem, suggest_similar_files, to_pascal_case, write_file};

fn main() {
    if let Err(e) = run() {
        eprintln!("{} {}", "error:".red().bold(), e);
        for cause in e.chain().skip(1) {
            eprintln!("  {} {}", "caused by:".dimmed(), cause);
        }
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = Args::parse();

    // Handle subcommands first
    if let Some(Command::Completions { shell }) = &args.command {
        let mut cmd = Args::command();
        clap_complete::generate(*shell, &mut cmd, "mold", &mut io::stdout());
        return Ok(());
    }

    // Setup color mode
    setup_color(&args);
    let is_tty = resolve_is_tty(&args);

    // Resolve output formats
    let (gen_ts, gen_zod, gen_prisma) = resolve_formats(&args)?;

    // Read input(s) - either files or stdin
    let inputs = read_inputs(&args)?;

    let config = GeneratorConfig {
        flat_mode: args.flat,
        indent: "  ".to_string(),
        ts_export_interfaces: args.ts_export,
        ts_readonly_fields: args.ts_readonly,
        zod_strict_objects: args.zod_strict,
        prisma_generate_relations: true,
    };

    let multi_file = inputs.len() > 1;

    for (json_content, root_name, source_path) in &inputs {
        let start = Instant::now();

        let schema = parse_json_string(json_content, &to_pascal_case(root_name), args.flat)
            .with_context(|| {
                match source_path {
                    Some(p) => format!("Failed to parse '{}'", p.display()),
                    None => "Failed to parse stdin".to_string(),
                }
            })?;

        // Verbose: print schema details
        if args.verbose {
            print_schema_details(&schema);
        }

        // Generate outputs
        let mut outputs: Vec<(&str, String, &str)> = Vec::new();

        if gen_ts {
            let gen = TypeScriptGenerator::new();
            let out = gen.generate(&schema, &config)?;
            outputs.push(("TypeScript", out, gen.file_extension()));
        }
        if gen_zod {
            let gen = ZodGenerator::new();
            let out = gen.generate(&schema, &config)?;
            outputs.push(("Zod", out, gen.file_extension()));
        }
        if gen_prisma {
            let gen = PrismaGenerator::new();
            let out = gen.generate(&schema, &config)?;
            outputs.push(("Prisma", out, gen.file_extension()));
        }

        let elapsed = start.elapsed();

        // Output results
        if let Some(output_dir) = &args.output {
            std::fs::create_dir_all(output_dir)
                .with_context(|| format!("Failed to create directory '{}'", output_dir.display()))?;

            let base_name = args
                .name
                .as_deref()
                .or_else(|| {
                    source_path
                        .as_ref()
                        .and_then(|p| p.file_stem())
                        .and_then(|s| s.to_str())
                })
                .unwrap_or("schema");

            for (name, content, ext) in &outputs {
                let file_path = output_dir.join(format!("{}.{}", base_name, ext));
                write_file(&file_path, content)?;
                if !args.quiet {
                    let size = format_size(content.len());
                    eprintln!(
                        "  {} {} {} {}",
                        "✓".green().bold(),
                        name,
                        format!("({})", size).dimmed(),
                        format!("→ {}", file_path.display()).dimmed()
                    );
                }
            }
        } else {
            // Stdout output
            if multi_file && is_tty && !args.quiet {
                if let Some(p) = source_path {
                    eprintln!("\n{}", format!("── {} ──", p.display()).dimmed());
                }
            }
            for (i, (name, content, _)) in outputs.iter().enumerate() {
                if is_tty && !args.quiet {
                    if i > 0 {
                        println!("\n{}", "─".repeat(60).dimmed());
                    }
                    println!("{}", format!("// {}", name).dimmed());
                } else if i > 0 {
                    println!();
                }
                if is_tty && !args.quiet {
                    println!("{}", highlight_output(content, name));
                } else {
                    print!("{}", content);
                }
            }
        }

        // Flush stdout before writing summary to stderr to prevent interleaving
        let _ = io::stdout().flush();

        // Summary (to stderr so it doesn't interfere with piped stdout)
        if !args.quiet {
            let (type_count, field_count) = count_stats(&schema);
            eprintln!(
                "\n  {} Generated {} {} with {} {} in {:.0?}",
                "Done.".green().bold(),
                type_count,
                if type_count == 1 { "type" } else { "types" },
                field_count,
                if field_count == 1 { "field" } else { "fields" },
                elapsed,
            );
        }
    }

    // Watch mode
    if args.watch {
        watch_loop(&args, &config, gen_ts, gen_zod, gen_prisma)?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Color & TTY
// ---------------------------------------------------------------------------

fn setup_color(args: &Args) {
    match args.color {
        ColorMode::Always => colored::control::set_override(true),
        ColorMode::Never => colored::control::set_override(false),
        ColorMode::Auto => {
            if !io::stdout().is_terminal() {
                colored::control::set_override(false);
            }
        }
    }
}

fn resolve_is_tty(args: &Args) -> bool {
    match args.color {
        ColorMode::Always => true,
        ColorMode::Never => false,
        ColorMode::Auto => io::stdout().is_terminal(),
    }
}

// ---------------------------------------------------------------------------
// Format resolution
// ---------------------------------------------------------------------------

fn resolve_formats(args: &Args) -> Result<(bool, bool, bool)> {
    let (ts, zod, prisma) = if args.all {
        (true, true, true)
    } else {
        (args.ts, args.zod, args.prisma)
    };

    if !ts && !zod && !prisma {
        let file_display = args
            .files
            .first()
            .map(|f| f.display().to_string())
            .unwrap_or_else(|| "<file>".to_string());
        return Err(MoldError::NoOutputFormat { file: file_display }.into());
    }

    Ok((ts, zod, prisma))
}

// ---------------------------------------------------------------------------
// Input reading (files + stdin)
// ---------------------------------------------------------------------------

/// Returns Vec of (json_content, root_name, optional_source_path)
fn read_inputs(args: &Args) -> Result<Vec<(String, String, Option<PathBuf>)>> {
    // No files provided — try stdin
    if args.files.is_empty() {
        if io::stdin().is_terminal() {
            return Err(anyhow::anyhow!(
                "No input file specified and stdin is a terminal\n  \
                 Try: mold <file.json> --ts\n  \
                 Or pipe JSON: cat data.json | mold --ts"
            ));
        }
        let mut input = String::new();
        io::stdin().read_to_string(&mut input)?;
        let name = args.name.clone().unwrap_or_else(|| "Root".to_string());
        return Ok(vec![(input, name, None)]);
    }

    let mut inputs = Vec::new();
    for path in &args.files {
        if path.as_os_str() == "-" {
            // Explicit stdin
            let mut input = String::new();
            io::stdin().read_to_string(&mut input)?;
            let name = args.name.clone().unwrap_or_else(|| "Root".to_string());
            inputs.push((input, name, None));
        } else {
            if !path.exists() {
                let hint = suggest_similar_files(path);
                return Err(anyhow::anyhow!(
                    "Cannot find file '{}'{}",
                    path.display(),
                    hint
                ));
            }
            let content = std::fs::read_to_string(path)
                .with_context(|| format!("Failed to read '{}'", path.display()))?;
            let name = args
                .name
                .clone()
                .unwrap_or_else(|| get_file_stem(path));
            inputs.push((content, name, Some(path.clone())));
        }
    }

    Ok(inputs)
}

// ---------------------------------------------------------------------------
// Stats
// ---------------------------------------------------------------------------

fn count_stats(schema: &Schema) -> (usize, usize) {
    let type_count = 1 + schema.nested_types.len();
    let field_count = match &schema.root_type {
        SchemaType::Object(obj) => obj.fields.len(),
        _ => 0,
    } + schema
        .nested_types
        .iter()
        .map(|nt| nt.object.fields.len())
        .sum::<usize>();
    (type_count, field_count)
}

fn format_size(bytes: usize) -> String {
    if bytes >= 1024 {
        format!("{:.1}KB", bytes as f64 / 1024.0)
    } else {
        format!("{}B", bytes)
    }
}

// ---------------------------------------------------------------------------
// Verbose output
// ---------------------------------------------------------------------------

fn print_schema_details(schema: &Schema) {
    eprintln!("{}", "Schema details:".cyan().bold());
    eprintln!("  Root: {}", schema.name.bold());
    if let SchemaType::Object(obj) = &schema.root_type {
        for field in &obj.fields {
            eprintln!(
                "    {} {}: {}{}",
                "·".dimmed(),
                field.name,
                format_type_name(&field.field_type),
                if field.optional { " (optional)".dimmed().to_string() } else { String::new() }
            );
        }
    }
    for nt in &schema.nested_types {
        eprintln!("  Nested: {}", nt.name.bold());
        for field in &nt.object.fields {
            eprintln!(
                "    {} {}: {}{}",
                "·".dimmed(),
                field.name,
                format_type_name(&field.field_type),
                if field.optional { " (optional)".dimmed().to_string() } else { String::new() }
            );
        }
    }
    eprintln!();
}

fn format_type_name(schema_type: &SchemaType) -> String {
    match schema_type {
        SchemaType::String => "String".to_string(),
        SchemaType::Number => "Number".to_string(),
        SchemaType::Integer => "Integer".to_string(),
        SchemaType::Boolean => "Boolean".to_string(),
        SchemaType::Null => "Null".to_string(),
        SchemaType::DateTime => "DateTime".yellow().to_string(),
        SchemaType::Date => "Date".yellow().to_string(),
        SchemaType::Uuid => "UUID".yellow().to_string(),
        SchemaType::Email => "Email".yellow().to_string(),
        SchemaType::Url => "URL".yellow().to_string(),
        SchemaType::Any => "Any".dimmed().to_string(),
        SchemaType::Array(inner) => format!("Array<{}>", format_type_name(inner)),
        SchemaType::Optional(inner) => format!("{}?", format_type_name(inner)),
        SchemaType::Union(types) => {
            let parts: Vec<String> = types.iter().map(format_type_name).collect();
            parts.join(" | ")
        }
        SchemaType::Enum(values) => format!("Enum({})", values.join(", ")),
        SchemaType::Object(obj) => {
            if obj.fields.is_empty() {
                "Object (empty)".to_string()
            } else {
                format!("Object ({} fields)", obj.fields.len())
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Syntax highlighting (simple, using colored crate)
// ---------------------------------------------------------------------------

fn highlight_output(code: &str, format_name: &str) -> String {
    code.lines()
        .map(|line| highlight_line(line, format_name))
        .collect::<Vec<_>>()
        .join("\n")
}

fn highlight_line(line: &str, format_name: &str) -> String {
    let trimmed = line.trim();

    // Comments
    if trimmed.starts_with("//") {
        return line.dimmed().to_string();
    }

    match format_name {
        "TypeScript" => highlight_ts_line(line),
        "Zod" => highlight_zod_line(line),
        "Prisma" => highlight_prisma_line(line),
        _ => line.to_string(),
    }
}

fn highlight_ts_line(line: &str) -> String {
    let trimmed = line.trim();
    if trimmed.starts_with("interface ") || trimmed.starts_with("export interface ") {
        return line
            .replace("export ", &"export ".magenta().to_string())
            .replace("interface ", &"interface ".cyan().bold().to_string());
    }
    if trimmed.starts_with("type ") || trimmed.starts_with("export type ") {
        return line
            .replace("export ", &"export ".magenta().to_string())
            .replace("type ", &"type ".cyan().bold().to_string());
    }
    line.to_string()
}

fn highlight_zod_line(line: &str) -> String {
    let trimmed = line.trim();
    if trimmed.starts_with("import ") {
        return line.dimmed().to_string();
    }
    if trimmed.starts_with("const ") {
        return line.replace("const ", &"const ".cyan().bold().to_string());
    }
    if trimmed.starts_with("type ") {
        return line.replace("type ", &"type ".cyan().bold().to_string());
    }
    if trimmed.starts_with("export ") {
        return line.replace("export ", &"export ".magenta().to_string());
    }
    line.to_string()
}

fn highlight_prisma_line(line: &str) -> String {
    let trimmed = line.trim();
    if trimmed.starts_with("model ") {
        return line.replace("model ", &"model ".cyan().bold().to_string());
    }
    if trimmed.contains("@id") || trimmed.contains("@default") || trimmed.contains("@unique") || trimmed.contains("@updatedAt") {
        // Highlight decorators
        let mut result = line.to_string();
        for attr in &["@id", "@default(autoincrement())", "@default(uuid())", "@default(now())", "@updatedAt", "@unique"] {
            if result.contains(attr) {
                result = result.replace(attr, &attr.yellow().to_string());
            }
        }
        return result;
    }
    line.to_string()
}

// ---------------------------------------------------------------------------
// Watch mode (polling)
// ---------------------------------------------------------------------------

fn watch_loop(
    args: &Args,
    config: &GeneratorConfig,
    gen_ts: bool,
    gen_zod: bool,
    gen_prisma: bool,
) -> Result<()> {
    let file_path = args
        .files
        .first()
        .filter(|p| p.as_os_str() != "-")
        .ok_or_else(|| anyhow::anyhow!("--watch requires a file path (not stdin)"))?;

    eprintln!(
        "\n  {} Watching {} for changes... (Ctrl+C to stop)",
        "⟳".cyan().bold(),
        file_path.display()
    );

    let mut last_modified = std::fs::metadata(file_path)?.modified()?;

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));

        let current_modified = match std::fs::metadata(file_path) {
            Ok(meta) => match meta.modified() {
                Ok(t) => t,
                Err(_) => continue,
            },
            Err(_) => continue,
        };

        if current_modified != last_modified {
            last_modified = current_modified;
            eprintln!("\n  {} File changed, regenerating...", "⟳".cyan().bold());

            let start = Instant::now();
            match regenerate(file_path, args, config, gen_ts, gen_zod, gen_prisma) {
                Ok((schema, outputs)) => {
                    // Write or print outputs
                    if let Some(output_dir) = &args.output {
                        let base_name = args
                            .name
                            .as_deref()
                            .or_else(|| file_path.file_stem().and_then(|s| s.to_str()))
                            .unwrap_or("schema");

                        for (name, content, ext) in &outputs {
                            let fp = output_dir.join(format!("{}.{}", base_name, ext));
                            write_file(&fp, content)?;
                            if !args.quiet {
                                let size = format_size(content.len());
                                eprintln!(
                                    "  {} {} {} {}",
                                    "✓".green().bold(),
                                    name,
                                    format!("({})", size).dimmed(),
                                    format!("→ {}", fp.display()).dimmed()
                                );
                            }
                        }
                    } else {
                        for (i, (_name, content, _)) in outputs.iter().enumerate() {
                            if i > 0 {
                                println!();
                            }
                            print!("{}", content);
                        }
                    }

                    let _ = io::stdout().flush();

                    if !args.quiet {
                        let elapsed = start.elapsed();
                        let (type_count, field_count) = count_stats(&schema);
                        eprintln!(
                            "  {} Regenerated {} {} with {} {} in {:.0?}",
                            "Done.".green().bold(),
                            type_count,
                            if type_count == 1 { "type" } else { "types" },
                            field_count,
                            if field_count == 1 { "field" } else { "fields" },
                            elapsed,
                        );
                    }
                }
                Err(e) => {
                    eprintln!("  {} {}", "error:".red().bold(), e);
                }
            }

            eprintln!(
                "  {} Watching for changes...",
                "⟳".cyan().bold()
            );
        }
    }
}

fn regenerate(
    file_path: &Path,
    args: &Args,
    config: &GeneratorConfig,
    gen_ts: bool,
    gen_zod: bool,
    gen_prisma: bool,
) -> Result<(Schema, Vec<(&'static str, String, &'static str)>)> {
    let content = std::fs::read_to_string(file_path)?;
    let root_name = args
        .name
        .clone()
        .unwrap_or_else(|| get_file_stem(file_path));
    let schema = parse_json_string(&content, &to_pascal_case(&root_name), args.flat)?;

    let mut outputs: Vec<(&'static str, String, &'static str)> = Vec::new();
    if gen_ts {
        let gen = TypeScriptGenerator::new();
        outputs.push(("TypeScript", gen.generate(&schema, config)?, gen.file_extension()));
    }
    if gen_zod {
        let gen = ZodGenerator::new();
        outputs.push(("Zod", gen.generate(&schema, config)?, gen.file_extension()));
    }
    if gen_prisma {
        let gen = PrismaGenerator::new();
        outputs.push(("Prisma", gen.generate(&schema, config)?, gen.file_extension()));
    }

    Ok((schema, outputs))
}
