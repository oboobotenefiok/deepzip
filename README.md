# deepzip

A fast, no-fuss CLI tool that extracts zip files into a **single flat directory**

I build this because I want to be able to send zip files easily to Deepseek on mobile without having to go through what I've been going through. I start with a simple version that just unzips it and puts in one folder but subsequent versions will filter out certain file extensions that aren't accepted by deepseek as well as create fifty (50) files per folder as per the file limit of deepseek per message - this will result in multiple folders but quite easier than before.

## What it does

Standard zip extraction preserves the internal folder structure of the archive, which can result in deeply nested directories. `deepzip` ("deep extraction zip") flattens everything: every file ends up directly in the output directory, regardless of how it was organized inside the zip.



- Strips all directory structure from extracted files
- Skips macOS metadata directories (`__MACOSX/`)
- Automatically renames duplicate filenames (`file_1.txt`, `file_2.txt`, etc.) instead of overwriting
- Preserves Unix file permissions
- Shows progress during extraction

## Installation

### Linux / macOS

```bash
# Build from source (requires Rust)
cargo build --release

# Install system-wide
sudo cp target/release/deepzip /usr/local/bin/

# Or install directly with Cargo
cargo install --path .
```

### Termux (Android)

```bash
# Install Rust
pkg install rust

# Clone or create the project
mkdir deepzip && cd deepzip

# Build and install to Termux's bin directory
cargo build --release
cp target/release/deepzip $PREFIX/bin/
```

## Usage

```bash
# Extract to a directory named after the zip file (default)
deepzip myfile.zip

# Extract to a custom output directory
deepzip myfile.zip --output /path/to/output

# Short flag
deepzip myfile.zip -o ./extracted
```

## Example

Given a zip with this structure:
```
project.zip
├── src/
│   ├── main.rs
│   └── utils.rs
├── docs/
│   └── readme.md
└── __MACOSX/   ← skipped automatically
```

Running `deepzip project.zip` produces:
```
project/
├── main.rs
├── utils.rs
└── readme.md
```

## Building from source

Requires [Rust](https://rustup.rs/) (stable).

```bash
git clone <repo-url>
cd deepzip
cargo build --release
```

The binary will be at `target/release/deepzip`.

## Dependencies

- [`clap`](https://crates.io/crates/clap) — argument parsing
- [`zip`](https://crates.io/crates/zip) — zip archive handling

## License

MIT
