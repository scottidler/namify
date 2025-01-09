use clap::Parser;
use env_logger::Env;
use eyre::{eyre, Result};
use glob::glob;
use log::{error, info};
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

/// A utility to prepend relative file paths as comments to the top of matching files.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Glob patterns to match files
    patterns: Vec<String>,
}

fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    // Parse command-line arguments
    let args = Args::parse();

    if args.patterns.is_empty() {
        error!("No glob patterns provided. Please specify at least one pattern.");
        return Err(eyre!("No glob patterns provided"));
    }

    let cwd = std::env::current_dir()?;
    let mut updated_files = Vec::new();

    // Process each glob pattern
    for pattern in &args.patterns {
        for entry in glob(pattern)? {
            match entry {
                Ok(path) => {
                    if process_file(&cwd, &path)? {
                        updated_files.push(path);
                    }
                }
                Err(e) => error!("Error processing glob pattern '{}': {}", pattern, e),
            }
        }
    }

    if !updated_files.is_empty() {
        info!("Updated files:");
        for file in updated_files {
            println!("{}", file.display());
        }
    }

    Ok(())
}

/// Processes a single file: checks for the relative filepath comment and adds it if not present.
/// Returns `true` if the file was updated, `false` otherwise.
fn process_file(cwd: &Path, file_path: &Path) -> Result<bool> {
    if !file_path.is_file() {
        return Ok(false);
    }

    let relative_path = file_path
        .strip_prefix(cwd)
        .unwrap_or(file_path)
        .to_string_lossy();
    let comment = format!("# {}\n", relative_path);

    // Open the file and check the first line
    let file = OpenOptions::new().read(true).open(file_path)?;
    let mut reader = BufReader::new(&file);
    let mut first_line = String::new();

    if reader.read_line(&mut first_line)? > 0 && first_line.trim() == comment.trim() {
        // File already has the relative path comment; no operation needed
        return Ok(false);
    }

    // Collect the rest of the file content
    let content: String = reader
        .lines()
        .map(|line| line.unwrap_or_default())
        .collect::<Vec<_>>()
        .join("\n");

    // Rewrite the file with the comment prepended
    let mut file = OpenOptions::new().write(true).truncate(true).open(file_path)?;
    file.write_all(comment.as_bytes())?;
    file.write_all(content.as_bytes())?;

    Ok(true)
}

