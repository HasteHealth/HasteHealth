//! Makes the descriptions that codegen turns into `#[doc]` attributes pass
//! `clippy::doc_markdown`: identifiers are wrapped in backticks and bare URLs
//! in angle brackets.
//!
//! Words are selected with the same heuristics as clippy
//! (`clippy_lints/src/doc/markdown.rs`), over the same markdown text events,
//! so already formatted code, links and code blocks are left alone and running
//! the rule twice changes nothing.

use super::{Edit, Rule};
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use serde_json::Value;
use std::ops::Range;

pub(crate) struct ClippyDocMarkdown;

impl Rule for ClippyDocMarkdown {
    fn name(&self) -> &'static str {
        "clippy-doc-markdown"
    }

    fn apply(&self, resource: &mut Value) -> Vec<Edit> {
        let mut edits = Vec::new();
        match resource.get("resourceType").and_then(Value::as_str) {
            // ElementDefinition.definition is the doc for generated structs and fields.
            Some("StructureDefinition") => {
                for view in ["snapshot", "differential"] {
                    let elements = format!("/{view}/element");
                    let count = resource
                        .pointer(&elements)
                        .and_then(Value::as_array)
                        .map_or(0, Vec::len);
                    for i in 0..count {
                        fix_field(resource, &format!("{elements}/{i}/definition"), &mut edits);
                    }
                }
            }
            // OperationDefinition.description and parameter documentation are the
            // docs for generated operation structs and fields.
            Some("OperationDefinition") => {
                fix_field(resource, "/description", &mut edits);
                fix_parameters(resource, "/parameter", &mut edits);
            }
            _ => {}
        }
        edits
    }
}

fn fix_parameters(resource: &mut Value, pointer: &str, edits: &mut Vec<Edit>) {
    let count = resource
        .pointer(pointer)
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    for i in 0..count {
        fix_field(resource, &format!("{pointer}/{i}/documentation"), edits);
        fix_parameters(resource, &format!("{pointer}/{i}/part"), edits);
    }
}

fn fix_field(resource: &mut Value, pointer: &str, edits: &mut Vec<Edit>) {
    let Some(value) = resource.pointer_mut(pointer) else {
        return;
    };
    let Some(fixed) = value.as_str().and_then(fix_markdown) else {
        return;
    };
    let before = std::mem::replace(value, Value::String(fixed));
    edits.push(Edit {
        pointer: pointer.to_string(),
        before,
        after: value.clone(),
    });
}

/// clippy's default `doc-valid-idents`: words that look like identifiers but are
/// accepted without backticks.
const VALID_IDENTS: &[&str] = &[
    "KiB",
    "MiB",
    "GiB",
    "TiB",
    "PiB",
    "EiB",
    "MHz",
    "GHz",
    "THz",
    "AccessKit",
    "CoAP",
    "CoreFoundation",
    "CoreGraphics",
    "CoreText",
    "DevOps",
    "Direct2D",
    "Direct3D",
    "DirectWrite",
    "DirectX",
    "ECMAScript",
    "GPLv2",
    "GPLv3",
    "GitHub",
    "GitLab",
    "IPv4",
    "IPv6",
    "InfiniBand",
    "RoCE",
    "ClojureScript",
    "CoffeeScript",
    "JavaScript",
    "PostScript",
    "PureScript",
    "TypeScript",
    "PowerPC",
    "PowerShell",
    "WebAssembly",
    "NaN",
    "NaNs",
    "OAuth",
    "GraphQL",
    "OCaml",
    "OpenAL",
    "OpenDNS",
    "OpenGL",
    "OpenMP",
    "OpenSSH",
    "OpenSSL",
    "OpenStreetMap",
    "OpenTelemetry",
    "OpenType",
    "WebGL",
    "WebGL2",
    "WebGPU",
    "WebRTC",
    "WebSocket",
    "WebTransport",
    "WebP",
    "OpenExr",
    "YCbCr",
    "sRGB",
    "TensorFlow",
    "TrueType",
    "iOS",
    "macOS",
    "FreeBSD",
    "NetBSD",
    "OpenBSD",
    "TeX",
    "LaTeX",
    "BibTeX",
    "BibLaTeX",
    "MinGW",
    "CamelCase",
];

/// Upper camel case with at least two capitals and one lowercase letter
/// (`Clippy` and `NASA` are fine), ignoring plurals (`IDs`).
fn is_camel_case(word: &str) -> bool {
    if word.starts_with(|c: char| c.is_ascii_digit() || c.is_ascii_lowercase()) {
        return false;
    }
    let all_upper = |s: &str| s.chars().all(|c| c.is_ascii_uppercase());
    let word = if let Some(prefix) = word.strip_suffix("es")
        && all_upper(prefix)
        && matches!(prefix.chars().last(), Some('S' | 'X'))
    {
        prefix
    } else if let Some(prefix) = word.strip_suffix("ified")
        && all_upper(prefix)
    {
        prefix
    } else {
        word.strip_suffix('s').unwrap_or(word)
    };

    word.chars().all(char::is_alphanumeric)
        && word.chars().filter(|c| c.is_uppercase()).take(2).count() > 1
        && word.chars().any(char::is_lowercase)
}

fn has_underscore(word: &str) -> bool {
    word != "_" && !word.contains("\\_") && word.contains('_')
}

fn has_hyphen(word: &str) -> bool {
    word != "-" && word.contains('-')
}

fn is_bare_url(word: &str) -> bool {
    // `foo::bar` parses as a URL too; clippy only flags URLs with a base.
    url::Url::parse(word).is_ok_and(|url| !url.cannot_be_a_base())
}

/// The replacement clippy would suggest for `word`, if any.
fn fix_word(word: &str) -> Option<String> {
    if word.contains(['`', '<', '>']) {
        return None;
    }
    if is_bare_url(word) {
        return Some(format!("<{word}>"));
    }
    // Mixed snake/kebab words are assumed not to be code.
    if has_underscore(word) && has_hyphen(word) {
        return None;
    }
    if has_underscore(word) || word.contains("::") || is_camel_case(word) || word.contains("()") {
        return Some(format!("`{word}`"));
    }
    None
}

/// Collects fixes for the words in `text`, which starts at `offset` in the
/// markdown source.
fn fix_text(text: &str, offset: usize, fixes: &mut Vec<(Range<usize>, String)>) {
    let trim = |c: char| !c.is_alphanumeric() && c != ':';

    for original in text.split(|c: char| c.is_whitespace() || c == '\'') {
        let mut word = original.trim_end_matches(trim);
        // Keep a directly following `()`, as in `foo()`.
        if let Some(with_parens) = original.get(..word.len() + 2)
            && with_parens.ends_with("()")
        {
            word = with_parens;
        }
        word = word.trim_start_matches(trim);
        // A single leading or trailing `:` is punctuation, `::` is a path.
        if word.starts_with(':') && !word.starts_with("::") {
            word = word.trim_start_matches(':');
        }
        if word.ends_with(':') && !word.ends_with("::") {
            word = word.trim_end_matches(':');
        }
        if word.is_empty() || word.chars().all(|c| c == ':') || VALID_IDENTS.contains(&word) {
            continue;
        }

        if let Some(replacement) = fix_word(word) {
            let start = offset + (word.as_ptr() as usize - text.as_ptr() as usize);
            fixes.push((start..start + word.len(), replacement));
        }
    }
}

/// Rewrites `source` so clippy's `doc_markdown` lints pass on it. Returns
/// `None` when nothing needs changing.
pub(crate) fn fix_markdown(source: &str) -> Option<String> {
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS;

    let mut fixes = Vec::new();
    let mut in_code_block = false;
    let mut code_tags = 0usize;
    let mut link_destination: Option<String> = None;
    // Adjacent text events are merged before splitting into words, as the
    // parser can split one word across several events.
    let mut run: Option<Range<usize>> = None;

    let flush = |run: &mut Option<Range<usize>>,
                 link_destination: &Option<String>,
                 fixes: &mut Vec<(Range<usize>, String)>| {
        if let Some(range) = run.take() {
            let text = &source[range.clone()];
            // A link whose text is its own URL is not a bare URL.
            if link_destination.as_deref() != Some(text) {
                fix_text(text, range.start, fixes);
            }
        }
    };

    for (event, range) in Parser::new_ext(source, options).into_offset_iter() {
        if let Event::Text(text) = &event {
            // Skip text inside code, and text the parser unescaped (`\_`,
            // `&amp;`), whose source offsets do not line up with its words.
            if in_code_block || code_tags > 0 || source.get(range.clone()) != Some(text.as_ref()) {
                flush(&mut run, &link_destination, &mut fixes);
                continue;
            }
            if let Some(current) = &mut run
                && current.end == range.start
            {
                current.end = range.end;
            } else {
                flush(&mut run, &link_destination, &mut fixes);
                run = Some(range);
            }
            continue;
        }

        flush(&mut run, &link_destination, &mut fixes);
        match event {
            Event::Start(Tag::CodeBlock(_)) => in_code_block = true,
            Event::End(TagEnd::CodeBlock) => in_code_block = false,
            Event::Start(Tag::Link { dest_url, .. }) => {
                link_destination = Some(dest_url.to_string());
            }
            Event::End(TagEnd::Link) => link_destination = None,
            Event::Html(html) | Event::InlineHtml(html) => {
                if html.starts_with("<code") {
                    code_tags += 1;
                } else if html.starts_with("</code") {
                    code_tags = code_tags.saturating_sub(1);
                }
            }
            _ => {}
        }
    }
    flush(&mut run, &link_destination, &mut fixes);

    if fixes.is_empty() {
        return None;
    }
    let mut rewritten = source.to_string();
    for (range, replacement) in fixes.into_iter().rev() {
        rewritten.replace_range(range, &replacement);
    }
    Some(rewritten)
}

#[cfg(test)]
mod tests {
    use super::fix_markdown;

    #[test]
    fn wraps_identifiers_and_urls() {
        assert_eq!(
            fix_markdown("A reference to a ClinicalImpression, see http://hl7.org/fhir.")
                .as_deref(),
            Some("A reference to a `ClinicalImpression`, see <http://hl7.org/fhir>.")
        );
        assert_eq!(
            fix_markdown("Use snake_case, foo::bar and run() here.").as_deref(),
            Some("Use `snake_case`, `foo::bar` and `run()` here.")
        );
    }

    #[test]
    fn leaves_accepted_words_alone() {
        for text in [
            "Plain words, NASA, IDs, Clippy and JavaScript.",
            "Codes like urn:oid:1.2.3 and mixed snake_and-kebab words.",
            "Note: the end.",
        ] {
            assert_eq!(fix_markdown(text), None, "{text}");
        }
    }

    #[test]
    fn skips_code_links_and_escapes() {
        for text in [
            "Already `CodeableConcept` formatted.",
            "```\nCodeBlock\n```",
            "See [http://hl7.org](http://hl7.org) or <http://hl7.org>.",
            "An <code>HtmlCode</code> tag.",
            "Escaped foo\\_bar.",
        ] {
            assert_eq!(fix_markdown(text), None, "{text}");
        }
    }

    #[test]
    fn is_idempotent() {
        let once = fix_markdown("The DocumentReference at https://example.org/a_b").unwrap();
        assert_eq!(fix_markdown(&once), None);
    }
}
