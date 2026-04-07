use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process;

mod analyze;
mod language;
mod loader;
mod pattern;
mod report;
mod scan;
mod spec;

#[derive(Parser)]
#[command(name = "idiom", about = "Local convention governance CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Infer and display local naming conventions for a directory
    Infer {
        /// Directory to analyze
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Output format: text (default) or json
        #[arg(long, default_value = "text")]
        format: String,
    },
    /// Check a file against the local conventions of its directory
    Check {
        /// File to check
        file: PathBuf,
        /// Output format: text (default) or json
        #[arg(long, default_value = "text")]
        format: String,
    },
    /// Emit a convention summary as structured context for agent injection
    Context {
        /// Directory to analyze
        #[arg(default_value = ".")]
        path: PathBuf,
        /// Output format: text (default) or json
        #[arg(long, default_value = "text")]
        format: String,
    },
}

fn main() {
    let cli = Cli::parse();
    let registry = language::LangRegistry::new();

    match cli.command {
        Command::Infer { path, format } => {
            if !path.is_dir() {
                eprintln!("Error: '{}' is not a directory", path.display());
                process::exit(1);
            }

            let summary = scan::infer_conventions(&path, &registry);

            if format == "json" {
                let json = report::render_json(&summary.patterns);
                println!("{json}");
            } else {
                print!("{}", report::render_summary(&summary));
            }
        }
        Command::Check { file, format } => {
            if !file.is_file() {
                eprintln!("Error: '{}' is not a file", file.display());
                process::exit(1);
            }

            let deviations = scan::check_file(&file, &registry);

            if format == "json" {
                let json = report::render_deviations_json(&deviations);
                println!("{json}");
                if !deviations.is_empty() {
                    process::exit(1);
                }
            } else if deviations.is_empty() {
                println!("idiom check passed.");
            } else {
                eprint!("{}", report::render_deviations(&deviations));
                eprintln!(
                    "error: aborting due to {} idiom deviation(s)",
                    deviations.len()
                );
                process::exit(1);
            }
        }
        Command::Context { path, format } => {
            if !path.is_dir() {
                eprintln!("Error: '{}' is not a directory", path.display());
                process::exit(1);
            }

            let summary = scan::infer_conventions(&path, &registry);

            if format == "json" {
                let json = report::render_json(&summary.patterns);
                println!("{json}");
            } else {
                print!("{}", report::render_context(&summary));
            }
        }
    }
}
