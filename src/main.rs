use clap::Parser;
use env_logger::Env;
use eyre::{eyre, Result};
use glob::glob;
use log::{error, info};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

mod built_info {
    include!(concat!(env!("OUT_DIR"), "/git_describe.rs"));
}

#[derive(Parser, Debug)]
#[command(author, version = built_info::GIT_DESCRIBE, about, long_about = None)]
struct Args {
    patterns: Vec<String>,
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let args = Args::parse();
    if args.patterns.is_empty() {
        error!("No glob patterns provided. Please specify at least one pattern.");
        return Err(eyre!("No glob patterns provided"));
    }

    let cwd = std::env::current_dir()?;
    let mut updated_files = Vec::new();

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

fn process_file(cwd: &Path, file_path: &Path) -> Result<bool> {
    if !file_path.is_file() {
        return Ok(false);
    }

    // Map of file extensions to comment styles
    let comment_styles: HashMap<&str, &str> = HashMap::from([
        ("rs", "//"), // Rust
        ("py", "#"),  // Python
        ("sh", "#"),  // Shell scripts
        ("c", "//"),  // C
        ("cpp", "//"), // C++
        ("js", "//"),  // JavaScript
        ("ts", "//"),  // TypeScript
        ("java", "//"), // Java
        ("yaml", "#"),  // YAML
        ("yml", "#"),   // YAML
    ]);

    // Determine the file extension and comment style
    let extension = file_path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
    let comment_prefix = comment_styles.get(extension).unwrap_or(&"#");
    let relative_path = file_path
        .strip_prefix(cwd)
        .unwrap_or(file_path)
        .to_string_lossy();
    let comment = format!("{} {}\n", comment_prefix, relative_path);

    // Open the file and process its content
    let file = OpenOptions::new().read(true).open(file_path)?;
    let mut reader = BufReader::new(&file);
    let mut first_line = String::new();

    // Check if the first line already contains the comment
    if reader.read_line(&mut first_line)? > 0 && first_line.trim() == comment.trim() {
        return Ok(false);
    }

    // Read the rest of the file
    let content: String = reader
        .lines()
        .map(|line| line.unwrap_or_default())
        .collect::<Vec<_>>()
        .join("\n");

    // Write the comment and original content back to the file
    let mut file = OpenOptions::new().write(true).truncate(true).open(file_path)?;
    file.write_all(comment.as_bytes())?;
    file.write_all(content.as_bytes())?;

    Ok(true)
}
