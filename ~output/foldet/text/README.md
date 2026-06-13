# deepzip

A CLI tool that unpacks a zip file and sorts its contents into folders DeepSeek's chat interface will accept. For formats DeepSeek does not accept but that contain readable text, it tries to convert them to `.txt` automatically.

## The problem

DeepSeek's chat UI does not accept zip files. If you unzip a project and try to upload everything, it will silently reject certain file types. And if your project is spread across dozens of nested folders, manually picking through them is tedious.

## What this does

You give it a zip and an output folder. It:

1. Extracts every file and classifies it by extension
2. Puts accepted files into named category folders
3. For unsupported-but-readable formats (RTF, EPUB, FB2), tries to extract the text and saves it as `.txt` in a `converted` folder
4. Puts true media and binary files in a `skipped` folder so nothing is silently lost

The output looks like this:

```
output/
  code/        <- source files (.rs, .py, .js, .go, .sql, ...)
  text/        <- plain text (.txt, .md, .csv, .log, .rst, ...)
  documents/   <- office docs DeepSeek reads (.pdf, .docx, .xlsx, .pptx, ...)
  images/      <- images DeepSeek accepts (.png, .jpg, .gif, .webp, .svg, ...)
  data/        <- config and markup (.json, .yaml, .toml, .xml, .html, .css, ...)
  converted/   <- RTF, EPUB, FB2 files converted to .txt
  skipped/     <- audio, video, executables, archives, fonts - nothing to extract
```

## Usage

```bash
deepzip input.zip output_folder
```

Example:

```bash
deepzip my_project.zip ./sorted
```

Sample output:

```
Opening  : my_project.zip
Output   : ./sorted

  [code] main.rs
  [code] lib.rs
  [data] Cargo.toml
  [text] README.md
  [documents] spec.pdf
  [converted] notes.rtf -> notes.txt
  [skipped] demo.mp4
  [skipped] app.exe

Done.
  Extracted  : 5
  Converted  : 1
  Skipped    : 2

  Converted files are in the 'converted' folder as .txt.
  Skipped files (media, binaries) are in the 'skipped' folder.
```

## Conversion support

| Format | How text is extracted |
|--------|----------------------|
| `.rtf` | State machine strips RTF control words |
| `.epub` | Opened as zip, XHTML chapters extracted and tags stripped |
| `.fb2`  | XML tags stripped, text content kept |

Formats that are **not** attempted for conversion (no useful text inside):
audio (`.mp3`, `.wav`, `.flac`), video (`.mp4`, `.mkv`, `.avi`), images, executables, compiled objects, fonts, nested archives.

## Install

You need Rust. If you do not have it:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Then clone and build:

```bash
git clone <this repo>
cd deepzip
cargo build --release
```

Copy the binary somewhere on your `PATH`:

```bash
cp target/release/deepzip ~/.local/bin/
```

## Running tests

```bash
cargo test
```

Tests live in `src/classifier.rs` and `src/converter.rs`.

## Adding file types

**New accepted format** (DeepSeek takes it natively): add the extension to the right arm in the `match` in `src/classifier.rs`.

**New convertible format** (has text inside, DeepSeek does not accept it): add the extension to the `Convertible` arm in `src/classifier.rs`, then add a handler branch in `src/converter.rs`.

**New skipped format** (media or binary, no text to extract): nothing to do, the wildcard `_ => Category::Skipped` already catches it.

## Project layout

```
deepzip/
  src/
    main.rs        - CLI parsing, calls extractor, prints summary
    classifier.rs  - extension -> Category (Code/Text/Documents/Images/Data/Convertible/Skipped)
    converter.rs   - bytes + extension -> Option<String> (extracted text)
    extractor.rs   - opens zip, routes each entry through classifier and converter
  Cargo.toml       - deps: zip, anyhow, clap, lopdf, calamine
  README.md
```

## Dependencies

| Crate | Why |
|-------|-----|
| `zip` | Reads zip archives and also parses EPUB files (which are zips internally) |
| `anyhow` | Clean error handling without custom error enum boilerplate |
| `clap` | CLI argument parsing with auto-generated help text |
| `lopdf` | PDF text extraction |
| `calamine` | Excel file reading (.xlsx, .xls, .ods) |

## Notes

- Nested directory structure inside the zip is flattened. `src/utils/helper.rs` becomes `helper.rs` in the `code` folder.
- Duplicate filenames get a counter suffix: `main.rs`, `main_1.rs`, `main_2.rs`.
- The tool does not recurse into nested zip files. A `.zip` inside your `.zip` goes to `skipped`.
- Conversion is best-effort. If an RTF or EPUB yields no text (empty, encrypted, corrupted), the original file is moved to `skipped` rather than producing a blank `.txt`.
