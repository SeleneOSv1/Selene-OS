#![forbid(unsafe_code)]

use unicode_normalization::UnicodeNormalization;

pub const NORM_VERSION: &str = "1.0.0";

const EXTRACTOR_MARKERS: &[&str] = &[
    "[[EXTRACTOR_START]]",
    "[[EXTRACTOR_END]]",
    "[[EXTRACTOR_MARKER]]",
    "<!--EXTRACTOR_START-->",
    "<!--EXTRACTOR_END-->",
    "<<EXTRACTOR>>",
];

pub fn normalize_document_for_chunking(input: &str) -> String {
    let normalized_newlines = normalize_newlines(input);
    let stripped_markers = strip_extractor_markers(&normalized_newlines);
    let unicode_nfc: String = stripped_markers.nfc().collect();
    let canonical_lines = normalize_whitespace_lines(&unicode_nfc);
    collapse_blank_lines(&canonical_lines)
}

pub fn normalize_newlines(input: &str) -> String {
    input.replace("\r\n", "\n").replace('\r', "\n")
}

pub fn strip_extractor_markers(input: &str) -> String {
    EXTRACTOR_MARKERS
        .iter()
        .fold(input.to_string(), |acc, marker| acc.replace(marker, ""))
}

pub fn normalize_whitespace_lines(input: &str) -> String {
    let mut out_lines = Vec::new();
    for line in input.lines() {
        let collapsed = line.split_whitespace().collect::<Vec<&str>>().join(" ");
        out_lines.push(collapsed.trim().to_string());
    }
    out_lines.join("\n")
}

fn collapse_blank_lines(input: &str) -> String {
    let mut out = Vec::new();
    let mut previous_blank = false;
    for line in input.lines() {
        if line.is_empty() {
            if !previous_blank {
                out.push(String::new());
            }
            previous_blank = true;
        } else {
            out.push(line.to_string());
            previous_blank = false;
        }
    }

    while out.first().is_some_and(|line| line.is_empty()) {
        out.remove(0);
    }
    while out.last().is_some_and(|line| line.is_empty()) {
        out.pop();
    }

    out.join("\n")
}

pub fn split_paragraphs(normalized: &str) -> Vec<String> {
    let mut paragraphs = Vec::new();
    let mut current = Vec::new();
    for line in normalized.lines() {
        if line.is_empty() {
            if !current.is_empty() {
                paragraphs.push(current.join(" "));
                current.clear();
            }
            continue;
        }
        current.push(line.trim().to_string());
    }
    if !current.is_empty() {
        paragraphs.push(current.join(" "));
    }
    paragraphs
}
