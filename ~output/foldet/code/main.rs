// Front door. Parse the arguments, hand off to the extractor, print what happened.
// No extraction logic lives here - that is the extractor's job.

mod classifier;
mod converter;
mod extractor;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "deepzip",
    about = "Unpacks a zip and sorts files into DeepSeek-friendly folders",
    long_about = "DeepSeek's chat UI does not accept zip files, and it rejects several \
                  other formats too. This tool unpacks your archive, puts accepted files \
                  into category folders, and tries to convert unsupported-but-readable \
                  formats (RTF, EPUB, FB2) to plain text before giving up on them."
)]
struct Args {
    // The zip file to unpack
    input: PathBuf,

    // The output folder to create and populate
    output: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if !args.input.exists() {
        anyhow::bail!("Cannot find the input file: {}", args.input.display());
    }

    if !args.input.extension().map_or(false, |ext| ext == "zip") {
        anyhow::bail!("Expected a .zip file, got: {}", args.input.display());
    }

    println!("Opening  : {}", args.input.display());
    println!("Output   : {}", args.output.display());
    println!();

    let summary = extractor::extract_and_sort(&args.input, &args.output)?;

    println!();
    println!("Done.");
    println!("  Extracted  : {}", summary.extracted);
    println!("  Converted  : {}", summary.converted);
    println!("  Skipped    : {}", summary.skipped);

    if summary.converted > 0 {
        println!();
        println!("  Converted files are in the 'converted' folder as .txt.");
    }
    if summary.skipped > 0 {
        println!("  Skipped files (media, binaries) are in the 'skipped' folder.");
    }

    Ok(())
}
