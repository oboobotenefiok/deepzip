mod extract_and_flatten;

//use for mod?

use std::{
    fs,
    path::{Path, PathBuf},
    io::{self, Write},
};
use zip::ZipArchive;
use clap::Parser;

use extract_and_flatten::extract_and_flatten;

/// Extract all contents from a zip file into a single folder
#[derive(Parser, Debug)]
#[command(name = "deepzip", author, version, about = "Extract zip files into a single flattened directory")]
struct Args {
    /// Zip file to extract (can be path or just filename)
    zip_file: PathBuf,
    
    /// Output directory (optional)
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    
    // Check if zip file exists
    if !args.zip_file.exists() {
        eprintln!(" Error: Zip file '{}' not found", args.zip_file.display());
        std::process::exit(1);
    }
    
    // Determine output directory (default: zip filename without extension)
    let output_dir = match args.output {
        Some(dir) => dir,
        None => args.zip_file.with_extension(""),
    };
    
    // Create output directory (clean if exists?)
    if output_dir.exists() {
        println!("Output directory exists, overwriting files...");
    } else {
        fs::create_dir_all(&output_dir)?;
    }
    
    // Extract and flatten
    match extract_and_flatten(&args.zip_file, &output_dir) {
        Ok(file_count) => {
            println!(" Successfully extracted {} files to: {}", file_count, output_dir.display());
        }
        Err(e) => {
            eprintln!(" Extraction failed: {}", e);
            std::process::exit(1);
        }
    }
    
    Ok(())
}

