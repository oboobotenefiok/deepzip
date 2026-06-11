fn extract_and_flatten(zip_path: &Path, output_dir: &Path) -> io::Result<usize> {
    let file = fs::File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;
    let mut extracted_count = 0;
    let mut used_names = std::collections::HashSet::new();
    
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name();
        
        // Skip directories and macOS junk files
        if name.ends_with('/') || name.starts_with("__MACOSX/") {
            continue;
        }
        
        // Get just the filename (flatten)
        let filename = Path::new(name)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        
        // Handle duplicate filenames
        let final_filename = if used_names.contains(&filename) {
            let stem = Path::new(&filename).file_stem().unwrap_or_default();
            let ext = Path::new(&filename).extension();
            let mut counter = 1;
            let mut new_name = filename.clone();
            
            while used_names.contains(&new_name) {
                new_name = match ext {
                    Some(e) => format!("{}_{}.{}", stem.to_string_lossy(), counter, e.to_string_lossy()),
                    None => format!("{}_{}", stem.to_string_lossy(), counter),
                };
                counter += 1;
            }
            new_name
        } else {
            filename.clone()
        };
        
        used_names.insert(final_filename.clone());
        let output_path = output_dir.join(&final_filename);
        
        // Extract file
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let mut outfile = fs::File::create(&output_path)?;
        io::copy(&mut file, &mut outfile)?;
        
        // Preserve permissions on Unix
        #[cfg(unix)]
        if let Some(mode) = file.unix_mode() {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&output_path, fs::Permissions::from_mode(mode))?;
        }
        
        extracted_count += 1;
        if extracted_count % 10 == 0 {
            print!("\r  Extracted: {} files...", extracted_count);
            io::stdout().flush()?;
        }
    }
    
    println!(); // New line after progress
    Ok(extracted_count)
}