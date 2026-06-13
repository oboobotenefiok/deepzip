// The classifier maps a filename to a destination category. The categories
// drive two decisions: which output folder a file lands in, and whether
// the extractor should attempt a text conversion before giving up on it.
//
// The key split in "skipped" territory is between:
//   - Convertible: has readable text locked inside a binary format (Word docs,
//     PDFs, spreadsheets, etc.) - worth running through the converter
//   - Skipped: genuinely opaque (audio, video, executables, compiled objects) -
//     no point trying, just move it to the skipped folder

/// Where a file ends up, and what the extractor should do with it.
#[derive(Debug, Clone, PartialEq)]
pub enum Category {
    // DeepSeek accepts these natively - each maps to a named output folder
    Code,
    Text,
    Documents,
    Images,
    Data,

    // Not accepted by DeepSeek, but we can try pulling the text out
    Convertible,

    // Not accepted, and there is no useful text to extract either
    Skipped,
}

impl Category {
    // The folder name this category writes into on disk
    pub fn folder_name(&self) -> &'static str {
        match self {
            Category::Code => "code",
            Category::Text => "text",
            Category::Documents => "documents",
            Category::Images => "images",
            Category::Data => "data",
            // Converted files land in "converted" so they are easy to find
            // and visually distinct from files that were already plain text
            Category::Convertible => "converted",
            Category::Skipped => "skipped",
        }
    }
}

/// Given a filename, decide what category it belongs to.
/// We only look at the extension - contents are never read here.
pub fn classify(filename: &str) -> Category {
    let ext = std::path::Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        // Source code in any language we know about
        "rs" | "py" | "js" | "ts" | "jsx" | "tsx" | "go" | "java" | "c" | "cpp" | "cc"
        | "cxx" | "h" | "hpp" | "cs" | "rb" | "php" | "swift" | "kt" | "kts" | "scala"
        | "r" | "lua" | "ex" | "exs" | "erl" | "hs" | "ml" | "mli" | "fs" | "fsx"
        | "clj" | "cljs" | "dart" | "zig" | "v" | "nim" | "sh" | "bash" | "zsh"
        | "fish" | "ps1" | "psm1" | "bat" | "cmd" | "asm" | "s" | "wasm" | "sql"
        | "graphql" | "gql" | "proto" | "thrift" | "dockerfile" | "makefile" => Category::Code,

        // Config, markup, and data formats that are already plain text
        "toml" | "yaml" | "yml" | "json" | "xml" | "html" | "htm" | "css" | "scss"
        | "sass" | "less" | "ini" | "cfg" | "conf" | "env" | "gitignore" | "gitattributes"
        | "editorconfig" | "eslintrc" | "prettierrc" | "babelrc" | "lock" => Category::Data,

        // Plain human-readable text in various formats
        "txt" | "md" | "markdown" | "rst" | "log" | "csv" | "tsv" | "tex" | "org" => {
            Category::Text
        }

        // Binary document formats DeepSeek will read directly
        "pdf" | "docx" | "doc" | "odt" | "pptx" | "ppt" | "xlsx" | "xls" | "ods" => {
            Category::Documents
        }

        // Images DeepSeek accepts
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "svg" | "ico" | "tiff" | "tif" => {
            Category::Images
        }

        // These are binary formats with text inside that we can extract.
        // They are things DeepSeek will not accept, but they are not media,
        // so it is worth converting them to .txt before giving up.
        // Note: doc/xls/ppt (old Office formats) are here because DeepSeek
        // will not accept them, unlike their modern docx/xlsx/pptx cousins.
        "rtf" | "epub" | "fb2" | "djvu" => Category::Convertible,

        // Everything from here down is genuinely opaque. Audio, video,
        // compiled binaries, archives, fonts - no text to extract.
        _ => Category::Skipped,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_goes_to_code() {
        assert_eq!(classify("main.rs"), Category::Code);
    }

    #[test]
    fn pdf_goes_to_documents() {
        // PDFs are accepted by DeepSeek directly, so they stay as documents
        assert_eq!(classify("report.pdf"), Category::Documents);
    }

    #[test]
    fn rtf_is_convertible() {
        assert_eq!(classify("notes.rtf"), Category::Convertible);
    }

    #[test]
    fn epub_is_convertible() {
        assert_eq!(classify("book.epub"), Category::Convertible);
    }

    #[test]
    fn exe_is_skipped() {
        assert_eq!(classify("program.exe"), Category::Skipped);
    }

    #[test]
    fn mp3_is_skipped() {
        assert_eq!(classify("song.mp3"), Category::Skipped);
    }

    #[test]
    fn mp4_is_skipped() {
        assert_eq!(classify("video.mp4"), Category::Skipped);
    }

    #[test]
    fn unknown_extension_is_skipped() {
        assert_eq!(classify("something.xyz"), Category::Skipped);
    }

    #[test]
    fn no_extension_is_skipped() {
        assert_eq!(classify("somebinary"), Category::Skipped);
    }

    #[test]
    fn uppercase_extension_works() {
        assert_eq!(classify("photo.PNG"), Category::Images);
    }

    #[test]
    fn toml_goes_to_data() {
        assert_eq!(classify("Cargo.toml"), Category::Data);
    }
}
