// This is the front door. It parses the command line, hands off to the
// extractor, and reports what happened. Nothing clever lives here.

mod classifier;
mod extractor;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

// The CLI shape: deepzip input.zip output_folder
// Clap will print nice help text and error messages for free.
#[derive(Parser, Debug)]
#[command(
    name = "deepzip",
    about = "Unpacks a zip file and sorts its contents into DeepSeek-friendly folders",
    long_about = "DeepSeek's chat UI does not accept zip files, and it silently rejects \
                  several other file types too. This tool unpacks your archive and groups \
                  the files by type into separate folders so you can upload them without \
                  guessing which ones will be accepted."
)]
struct Args {
    // The zip file you want to unpack
    input: PathBuf,

    // The folder that will be created (or populated) with the sorted output
    output: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Make sure the input file actually exists before we try anything else.
    // Giving a clear error here saves the user from a confusing message later.
    if !args.input.exists() {
        anyhow::bail!(
            "Cannot find the input file: {}",
            args.input.display()
        );
    }

    if !args.input.extension().map_or(false, |ext| ext == "zip") {
        anyhow::bail!(
            "Expected a .zip file, but got: {}",
            args.input.display()
        );
    }

    println!("Opening: {}", args.input.display());
    println!("Output folder: {}", args.output.display());

    let summary = extractor::extract_and_sort(&args.input, &args.output)?;

    println!("\nDone.");
    println!("  Files extracted : {}", summary.extracted);
    println!("  Files skipped   : {}", summary.skipped);

    if summary.skipped > 0 {
        println!(
            "\n  Skipped files are types DeepSeek does not accept (see the 'skipped' folder)."
        );
    }

    Ok(())
}
