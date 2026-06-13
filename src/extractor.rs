// The extractor does the actual work: open the archive, walk every entry,
// classify it, and write it out to the right folder. It also handles the
// awkward edge cases - duplicate filenames, nested directories inside the
// zip, and entries that are directories themselves (which we skip).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use zip::ZipArchive;

use crate::classifier;

/// A quick summary of what happened during extraction, printed at the end.
pub struct ExtractionSummary {
    pub extracted: usize,
    pub skipped: usize,
}

/// Opens the zip at `input`, creates `output` if it does not exist, and
/// sorts every file entry into the appropriate subfolder.
pub fn extract_and_sort(input: &Path, output: &Path) -> Result<ExtractionSummary> {
    let zip_file = fs::File::open(input)
        .with_context(|| format!("Could not open zip file: {}", input.display()))?;

    let mut archive = ZipArchive::new(zip_file)
        .with_context(|| format!("Could not read zip archive: {}", input.display()))?;

    // Create the output directory now so we fail early if something is wrong
    // with the path (permissions, bad characters, etc.)
    fs::create_dir_all(output)
        .with_context(|| format!("Could not create output directory: {}", output.display()))?;

    let mut extracted = 0usize;
    let mut skipped = 0usize;

    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .with_context(|| format!("Could not read entry at index {}", index))?;

        // Directories inside the zip are just structural noise for our purposes.
        // We flatten everything into category folders, so we can skip them.
        if entry.is_dir() {
            continue;
        }

        // Grab just the filename, stripping any directory path the zip entry
        // might have. We want "src/main.rs" to become "main.rs" in the output.
        let raw_name = entry.name().to_string();
        let filename = bare_filename(&raw_name);

        // An empty filename after stripping is unusual but possible with
        // malformed archives. Skip rather than panic.
        if filename.is_empty() {
            println!("  Skipping entry with no usable filename: {}", raw_name);
            continue;
        }

        let category = classifier::classify(filename);
        let category_dir = output.join(category.folder_name());

        fs::create_dir_all(&category_dir).with_context(|| {
            format!(
                "Could not create category directory: {}",
                category_dir.display()
            )
        })?;

        // If two files in the zip have the same name, we do not want to
        // silently overwrite one. Append a counter until the name is free.
        let dest_path = unique_path(&category_dir, filename);

        write_entry(&mut entry, &dest_path).with_context(|| {
            format!("Failed to write file: {}", dest_path.display())
        })?;

        println!(
            "  [{}] {}",
            category.folder_name(),
            filename
        );

        if category == classifier::Category::Skipped {
            skipped += 1;
        } else {
            extracted += 1;
        }
    }

    Ok(ExtractionSummary { extracted, skipped })
}

// Takes a zip entry name like "project/src/main.rs" and returns just "main.rs".
// If the name ends with a slash (a directory entry that slipped through), this
// returns an empty string, which the caller checks for.
fn bare_filename(entry_name: &str) -> &str {
    // Path::file_name gives us the last component, which is exactly what we want.
    Path::new(entry_name)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
}

// If "output/code/main.rs" already exists, try "main_1.rs", "main_2.rs", and
// so on until we find a name that is free. This is simple and predictable.
fn unique_path(dir: &Path, filename: &str) -> PathBuf {
    let candidate = dir.join(filename);
    if !candidate.exists() {
        return candidate;
    }

    // Split "main.rs" into ("main", "rs") so we can insert the counter between them.
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

    // Realistically we will never get here, but the compiler needs us to
    // return something after the loop.
    dir.join(filename)
}

// Copies the bytes from a zip entry out to a file on disk.
// Using io::copy is the right move here - it streams in chunks and does not
// load the whole file into memory at once, which matters for large archives.
fn write_entry<R: io::Read>(entry: &mut R, dest: &Path) -> Result<()> {
    let mut out = fs::File::create(dest)
        .with_context(|| format!("Could not create output file: {}", dest.display()))?;

    io::copy(entry, &mut out)
        .with_context(|| format!("Failed while writing: {}", dest.display()))?;

    Ok(())
}
