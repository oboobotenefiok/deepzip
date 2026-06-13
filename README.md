# deepzip

A small CLI tool that unpacks a zip file and sorts its contents into folders that DeepSeek's chat interface will actually accept.


## The problem

DeepSeek's chat UI does not accept zip files. If you unzip a project and try to upload everything, it will silently reject certain file types anyway. And if your project is spread across dozens of nested folders, manually picking through them to find the uploadable files is a pain.

## What this does

You hand it a zip and tell it where to put the output. It extracts every file, classifies it by extension, and writes it into a named subfolder. You end up with something like:

```
output/
  code/        <- .rs, .py, .js, .go, .sql, and everything else that is source
  text/        <- .txt, .md, .csv, .log, .rst
  documents/   <- .pdf, .docx, .xlsx, .pptx, .odt
  images/      <- .png, .jpg, .gif, .webp, .svg, .ico
  data/        <- .json, .yaml, .toml, .xml, .html, .css, .env, .lock
  skipped/     <- anything DeepSeek will not accept (videos, binaries, etc.)
```

Nothing is thrown away. Files DeepSeek cannot handle go into `skipped` so you still have them.

## Usage

I simply assume you're using a Linux environment, otherwise just figure it out.

```bash
deepzip input.zip output_folder
```

Example:

```bash
deepzip my_project.zip ./sorted
```

If `sorted` does not exist it will be created. If two files in the archive have the same name, the second one is renamed automatically (`main_1.rs`, `main_2.rs`, etc.) so nothing is silently overwritten.

## Install

You need Rust. If you do not have it:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Then clone and build:

```bash
git clone https://github.com/oboobotenefiok/deepzip
cd deepzip
cargo build --release
```

The binary ends up at `target/release/deepzip`. You can copy it somewhere on your `PATH`:

```bash
cp target/release/deepzip ~/.local/bin/
```

Or run it directly:

```bash
./target/release/deepzip input.zip output_folder
```

If you want to build and run in one step, do:

```bash
cargo run input.zip output_folder
```
  
## Running tests

```bash
cargo test
```

The tests live in `src/classifier.rs` and cover the common cases: known extensions, unknown extensions, uppercase extensions, and files with no extension at all.

## Adding file types

Everything is in `src/classifier.rs` inside the `classify` function. It is a single `match` on the lowercased extension. Adding a new type means adding its extension string to the right arm. It should take about ten seconds.

## Project layout

```
deepzip/
  src/
    main.rs        - CLI argument parsing, calls extractor, prints summary
    classifier.rs  - maps file extensions to category buckets
    extractor.rs   - opens the zip, walks entries, writes files to disk
  Cargo.toml       - dependencies: zip, anyhow, clap
  README.md        - you are here
```

## Dependencies

| Crate | Why |
|-------|-----|
| `zip` | Reads zip archives entry by entry without loading everything into memory |
| `anyhow` | Error handling that gives useful messages without a lot of boilerplate |
| `clap` | CLI argument parsing with automatic help text generation |

## Notes

- Nested directory structure inside the zip is flattened. `src/utils/helper.rs` becomes `helper.rs` in the `code` folder.
- KINDLY NOTE THAT the tool does not recurse into nested zip files. A `.zip` inside your `.zip` goes to `skipped`. To unzip that, you'll have to run the program again and pass the nested zip(now in the skipped folder) as the new argument.
- ALSO NOTE: File contents are never inspected, only extensions. A file named `notes.txt` containing a Python script will go to `text`, not `code`. Rename it if that matters.

Feel free to respond with issues and your pull requests. Try as much as possible to keep your contributions in the Rust Programming Language. I may create a detailed CONTRIBUTING.md someday but it's ta-ta for now.

With love,

- Obot
