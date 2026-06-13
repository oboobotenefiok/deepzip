// The extractor opens the archive, walks every entry, and routes each file
// to the right place. For files that DeepSeek does not accept but that might
// contain readable text, it tries a conversion before falling back to skipped.
//
// The flow for each zip entry is:
//   1. Classify by extension
//   2. If Convertible -> read bytes -> try_convert -> write .txt, or fall to skipped
//   3. Otherwise -> write the file as-is into the right category folder

use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use zip::ZipArchive;

use crate::classifier::{self, Category};
use crate::converter;

/// Tallies for the summary line printed at the end of a run.
pub struct ExtractionSummary {
    pub extracted: usize,   // went straight to a category folder
    pub converted: usize,   // was converted to .txt
    pub skipped: usize,     // could not be used or converted
}

/// The main entry point. Opens the zip, processes every file entry,
/// and writes results into subfolders of `output`.
pub fn extract_and_sort(input: &Path, output: &Path) -> Result<ExtractionSummary> {
    let zip_file = fs::File::open(input)
        .with_context(|| format!("Could not open zip file: {}", input.display()))?;

    let mut archive = ZipArchive::new(zip_file)
        .with_context(|| format!("Could not read zip archive: {}", input.display()))?;

    fs::create_dir_all(output)
        .with_context(|| format!("Could not create output directory: {}", output.display()))?;

    let mut extracted = 0usize;
    let mut converted = 0usize;
    let mut skipped   = 0usize;

    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .with_context(|| format!("Could not read zip entry at index {}", index))?;

        if entry.is_dir() {
            continue;
        }

        let raw_name = entry.name().to_string();
        let filename = bare_filename(&raw_name);

        if filename.is_empty() {
            println!("  Skipping entry with no usable filename: {}", raw_name);
            continue;
        }

        let ext = file_extension(filename);
        let category = classifier::classify(filename);

        match category {
            Category::Convertible => {
                // Read the whole entry into memory so the converter can work with it.
                // We accept the memory cost here because conversion formats (RTF, EPUB)
                // are almost never gigantic.
                let mut bytes = Vec::new();
                entry.read_to_end(&mut bytes)
                    .with_context(|| format!("Could not read bytes from: {}", filename))?;

                match converter::try_convert(&ext, &bytes) {
                    Some(text) => {
                        // The file had readable content. Write it as .txt in the
                        // converted folder so it is easy to find and upload.
                        let txt_name = format!(
                            "{}.txt",
                            Path::new(filename)
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or(filename)
                        );
                        let converted_dir = output.join("converted");
                        fs::create_dir_all(&converted_dir)?;
                        let dest = unique_path(&converted_dir, &txt_name);
                        fs::write(&dest, text.as_bytes())
                            .with_context(|| format!("Could not write converted file: {}", dest.display()))?;
                        println!("  [converted] {} -> {}", filename, txt_name);
                        converted += 1;
                    }
                    None => {
                        // Conversion came up empty. Move it to skipped so it
                        // is not silently lost, just not useful for DeepSeek.
                        let skipped_dir = output.join("skipped");
                        fs::create_dir_all(&skipped_dir)?;
                        let dest = unique_path(&skipped_dir, filename);
                        fs::write(&dest, &bytes)
                            .with_context(|| format!("Could not write skipped file: {}", dest.display()))?;
                        println!("  [skipped] {} (conversion yielded nothing)", filename);
                        skipped += 1;
                    }
                }
            }

            Category::Skipped => {
                // True media or binary - copy as-is to the skipped folder.
                // We do not attempt conversion because there is nothing to extract.
                let skipped_dir = output.join("skipped");
                fs::create_dir_all(&skipped_dir)?;
                let dest = unique_path(&skipped_dir, filename);
                write_entry(&mut entry, &dest)
                    .with_context(|| format!("Failed to write skipped file: {}", dest.display()))?;
                println!("  [skipped] {}", filename);
                skipped += 1;
            }

            // For everything DeepSeek accepts natively, just copy it into the
            // right category folder and move on.
            ref accepted => {
                let category_dir = output.join(accepted.folder_name());
                fs::create_dir_all(&category_dir)?;
                let dest = unique_path(&category_dir, filename);
                write_entry(&mut entry, &dest)
                    .with_context(|| format!("Failed to write file: {}", dest.display()))?;
                println!("  [{}] {}", accepted.folder_name(), filename);
                extracted += 1;
            }
        }
    }

    Ok(ExtractionSummary { extracted, converted, skipped })
}

// Returns the lowercased extension of a filename, or an empty string if there
// is none. We lowercase so the converter does not need to handle both "RTF"
// and "rtf" variants.
fn file_extension(filename: &str) -> String {
    Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase()
}

// Strips any leading directory path from a zip entry name.
// "src/utils/helper.rs" becomes "helper.rs".
// A name that ends with "/" or has no final component returns "".
fn bare_filename(entry_name: &str) -> &str {
    Path::new(entry_name)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
}

// Finds a path in `dir` that does not already exist by appending a counter.
// "main.rs" -> "main_1.rs" -> "main_2.rs" etc.
// This prevents silent overwrites when two zip entries share a filename.
fn unique_path(dir: &Path, filename: &str) -> PathBuf {
    let candidate = dir.join(filename);
    if !candidate.exists() {
        return candidate;
    }

    let stem = Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(filename);

    let ext = Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{}", e))
        .unwrap_or_default();

    for counter in 1u32.. {
        let name = format!("{}_{}{}", stem, counter, ext);
        let path = dir.join(&name);
        if !path.exists() {
            return path;
        }
    }

    dir.join(filename)
}

// Streams bytes from a zip entry to a file on disk.
// io::copy works in chunks so large files do not blow up memory.
fn write_entry<R: io::Read>(entry: &mut R, dest: &Path) -> Result<()> {
    let mut out = fs::File::create(dest)
        .with_context(|| format!("Could not create output file: {}", dest.display()))?;
    io::copy(entry, &mut out)
        .with_context(|| format!("Failed while writing: {}", dest.display()))?;
    Ok(())
}
