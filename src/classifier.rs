// The classifier is the brain of the sorting logic. Given a filename, it
// decides which bucket that file belongs in, and whether DeepSeek will
// accept it at all.
//
// DeepSeek's chat interface accepts: plain text, code files, PDFs, images,
// and a handful of document formats. Anything else (videos, executables,
// archives, etc.) gets moved to a "skipped" folder rather than thrown away,
// so you can still find them if you need them.

/// Where a file ends up after classification.
#[derive(Debug, Clone, PartialEq)]
pub enum Category {
    // DeepSeek accepts these - each maps to a folder in the output directory
    Code,
    Text,
    Documents,
    Images,
    Data,

    // DeepSeek will not accept these, but we keep them so nothing gets lost
    Skipped,
}

impl Category {
    // The folder name this category maps to on disk
    pub fn folder_name(&self) -> &'static str {
        match self {
            Category::Code => "code",
            Category::Text => "text",
            Category::Documents => "documents",
            Category::Images => "images",
            Category::Data => "data",
            Category::Skipped => "skipped",
        }
    }
}

/// Looks at the file extension and returns the right category.
/// If there is no extension, or we do not recognise it, the file goes to skipped.
pub fn classify(filename: &str) -> Category {
    // Pull the extension out and lowercase it so we are not doing
    // case-sensitive comparisons against every variant.
    let ext = std::path::Path::new(filename)
        .extension()
        .and_then(|e| e.to_str()) 
        .unwrap_or("")
        .to_lowercase();
//I would've loved to handle error by propagation here but I'll leave it for now, just for my sanity. Just have to trust those methods won't crash anything.
    match ext.as_str() {
        // Source code - the long list is intentional. DeepSeek handles most
        // languages well and people often have mixed-language projects in a zip.
        "rs" | "py" | "js" | "ts" | "jsx" | "tsx" | "go" | "java" | "c" | "cpp" | "cc"
        | "cxx" | "h" | "hpp" | "cs" | "rb" | "php" | "swift" | "kt" | "kts" | "scala"
        | "r" | "lua" | "ex" | "exs" | "erl" | "hs" | "ml" | "mli" | "fs" | "fsx"
        | "clj" | "cljs" | "dart" | "zig" | "v" | "nim" | "sh" | "bash" | "zsh"
        | "fish" | "ps1" | "psm1" | "bat" | "cmd" | "asm" | "s" | "wasm" | "sql"
        | "graphql" | "gql" | "proto" | "thrift" | "dockerfile" | "makefile" => Category::Code,

        // Config and markup files that are still plaintext and readable by DeepSeek
        "toml" | "yaml" | "yml" | "json" | "xml" | "html" | "htm" | "css" | "scss"
        | "sass" | "less" | "ini" | "cfg" | "conf" | "env" | "gitignore" | "gitattributes"
        | "editorconfig" | "eslintrc" | "prettierrc" | "babelrc" | "lock" => Category::Data,

        // Plain readable text
        "txt" | "md" | "markdown" | "rst" | "log" | "csv" | "tsv" | "tex" | "org" => {
            Category::Text
        }

        // Documents DeepSeek can read
        "pdf" | "docx" | "doc" | "odt" | "pptx" | "ppt" | "xlsx" | "xls" | "ods" => {
            Category::Documents
        }

        // Images DeepSeek accepts
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "svg" | "ico" | "tiff" | "tif" => {
            Category::Images
        }

        // Everything else - videos, binaries, archives-within-archives, fonts,
        // compiled objects, etc. DeepSeek will not take these.
        _ => Category::Skipped,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rust_file_goes_to_code() {
        assert_eq!(classify("main.rs"), Category::Code);
    }

    #[test]
    fn pdf_goes_to_documents() {
        assert_eq!(classify("report.pdf"), Category::Documents);
    }

    #[test]
    fn exe_goes_to_skipped() {
        assert_eq!(classify("program.exe"), Category::Skipped);
    }

    #[test]
    fn no_extension_goes_to_skipped() {
        // A file with no extension that we do not recognise should be isolated
        assert_eq!(classify("somebinary"), Category::Skipped);
    }

    #[test]
    fn uppercase_extension_is_handled() {
        assert_eq!(classify("photo.PNG"), Category::Images);
    }

    #[test]
    fn toml_goes_to_data() {
        assert_eq!(classify("Cargo.toml"), Category::Data);
    }
}
