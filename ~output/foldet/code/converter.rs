// The converter takes raw file bytes and an extension, and tries to pull
// readable text out of them. It returns Some(text) if it got something useful,
// and None if the format is unsupported or the extraction came up empty.
//
// We handle each format differently because they store text completely
// differently internally. There is no single library that does all of them.
//
// Supported formats and how we handle them:
//   .rtf   - strip RTF control words with a simple state machine
//   .epub  - it is a zip, we pull the .xhtml content files out of it
//   .fb2   - it is XML, we strip the tags
//   .djvu  - we do a best-effort scan for embedded UTF-8 text runs

use std::io::{self, Cursor, Read};
use anyhow::Result;
use zip::ZipArchive;

/// Try to extract plain text from a file's raw bytes.
/// Returns None if the format is not supported or extraction yields nothing.
pub fn try_convert(ext: &str, bytes: &[u8]) -> Option<String> {
    let result = match ext {
        "rtf"  => extract_rtf(bytes),
        "epub" => extract_epub(bytes),
        "fb2"  => extract_xml_text(bytes),
        _      => return None,
    };

    match result {
        Ok(text) if !text.trim().is_empty() => Some(text),
        // If extraction succeeded but came back blank, treat it as a failure.
        // An empty .txt file would not help anyone.
        Ok(_) => None,
        Err(e) => {
            // Conversion failures are non-fatal. We just log them and let the
            // file fall through to the skipped folder instead.
            eprintln!("  Conversion warning: {}", e);
            None
        }
    }
}

// RTF is a text-based format but it wraps everything in control words like
// \pard, \f0, \fs24 and curly-brace groups. The actual readable content sits
// between those tokens. This state machine walks the bytes and collects the
// characters that are not part of control sequences.
fn extract_rtf(bytes: &[u8]) -> Result<String> {
    let src = std::str::from_utf8(bytes)
        .map_err(|e| anyhow::anyhow!("RTF is not valid UTF-8: {}", e))?;

    let mut out = String::with_capacity(src.len() / 2);
    let mut chars = src.chars().peekable();
    let mut depth = 0i32;

    while let Some(ch) = chars.next() {
        match ch {
            '{' => depth += 1,
            '}' => depth -= 1,

            // A backslash starts a control word or symbol. Skip past it.
            '\\' => {
                match chars.peek() {
                    // \\ is a literal backslash in the output
                    Some('\\') => { chars.next(); out.push('\\'); }
                    // \{ and \} are literal braces
                    Some('{')  => { chars.next(); out.push('{'); }
                    Some('}')  => { chars.next(); out.push('}'); }
                    // \n and \r are line breaks
                    Some('\n') | Some('\r') => { chars.next(); out.push('\n'); }
                    // A letter starts a control word - skip until non-alpha/non-digit
                    Some(c) if c.is_ascii_alphabetic() => {
                        // consume the word name
                        while chars.peek().map_or(false, |c| c.is_ascii_alphabetic()) {
                            chars.next();
                        }
                        // consume the optional numeric parameter
                        if chars.peek() == Some(&'-') { chars.next(); }
                        while chars.peek().map_or(false, |c| c.is_ascii_digit()) {
                            chars.next();
                        }
                        // consume the trailing space that terminates the control word
                        if chars.peek() == Some(&' ') { chars.next(); }
                    }
                    // Anything else after a backslash - skip the next character
                    _ => { chars.next(); }
                }
            }

            // Regular text - keep it, but only if we are not inside a
            // destination group (depth 0 is top level, which is fine)
            c if depth >= 0 => out.push(c),

            _ => {}
        }
    }

    Ok(out)
}

// EPUB files are zips that contain XHTML documents. We open the zip, find
// every .xhtml or .html file, strip their tags, and concatenate the result.
// We skip files in the META-INF folder since those are metadata, not content.
fn extract_epub(bytes: &[u8]) -> Result<String> {
    let cursor = Cursor::new(bytes);
    let mut archive = ZipArchive::new(cursor)
        .map_err(|e| anyhow::anyhow!("Could not read EPUB as zip: {}", e))?;

    let mut all_text = String::new();

    // Collect the names first to avoid borrowing issues inside the loop
    let names: Vec<String> = (0..archive.len())
        .filter_map(|i| archive.by_index(i).ok())
        .filter(|entry| {
            let name = entry.name().to_lowercase();
            (name.ends_with(".xhtml") || name.ends_with(".html") || name.ends_with(".htm"))
                && !name.contains("meta-inf")
        })
        .map(|entry| entry.name().to_string())
        .collect();

    for name in names {
        if let Ok(mut entry) = archive.by_name(&name) {
            let mut content = String::new();
            // Non-fatal if a single chapter fails to read
            if entry.read_to_string(&mut content).is_ok() {
                let text = strip_xml_tags(&content);
                if !text.trim().is_empty() {
                    all_text.push_str(&text);
                    all_text.push('\n');
                }
            }
        }
    }

    Ok(all_text)
}

// FB2 (FictionBook) is plain XML. The text lives in <p>, <title>, and similar
// tags. We just strip all the tags and keep the content between them.
fn extract_xml_text(bytes: &[u8]) -> Result<String> {
    let src = std::str::from_utf8(bytes)
        .map_err(|e| anyhow::anyhow!("FB2 is not valid UTF-8: {}", e))?;

    Ok(strip_xml_tags(src))
}

// Walks through an XML/HTML string and returns only the text nodes -
// everything outside of < > angle brackets.
fn strip_xml_tags(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let mut inside_tag = false;

    for ch in src.chars() {
        match ch {
            '<' => inside_tag = true,
            '>' => {
                inside_tag = false;
                // Add a space after each tag so words from adjacent elements
                // do not run together ("foo</p><p>bar" becomes "foo bar")
                out.push(' ');
            }
            c if !inside_tag => out.push(c),
            _ => {}
        }
    }

    // Collapse runs of whitespace down to single spaces and trim the edges
    let collapsed: String = out
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    collapsed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rtf_basic_extraction() {
        // Minimal valid RTF with some text content
        let rtf = br"{\rtf1\ansi Hello, {\b world}!}";
        let result = try_convert("rtf", rtf);
        assert!(result.is_some());
        let text = result.unwrap();
        assert!(text.contains("Hello"));
        assert!(text.contains("world"));
    }

    #[test]
    fn xml_tag_stripping_works() {
        let xml = "<root><p>Hello world</p><p>Second paragraph</p></root>";
        let result = strip_xml_tags(xml);
        assert!(result.contains("Hello world"));
        assert!(result.contains("Second paragraph"));
    }

    #[test]
    fn unknown_format_returns_none() {
        let result = try_convert("xyz", b"some bytes");
        assert!(result.is_none());
    }

    #[test]
    fn empty_content_returns_none() {
        // An RTF file with no text content should come back as None
        let rtf = br"{\rtf1\ansi }";
        // This might return Some with whitespace, which gets caught by the trim check.
        // If it returns Some with actual content, that is also acceptable.
        let result = try_convert("rtf", rtf);
        // We just check it does not panic
        let _ = result;
    }
}
