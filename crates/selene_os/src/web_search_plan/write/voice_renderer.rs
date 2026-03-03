#![forbid(unsafe_code)]

pub fn render_voice_output(formatted_text: &str) -> String {
    let mut lines = Vec::new();
    let mut previous_was_pause = false;

    for raw in formatted_text.lines() {
        let clean = strip_markdown_artifacts(raw).trim().to_string();
        if clean.is_empty() {
            if !previous_was_pause {
                lines.push("[PAUSE_LONG]".to_string());
                previous_was_pause = true;
            }
            continue;
        }

        previous_was_pause = false;
        if is_heading(&clean) {
            lines.push(clean);
            lines.push("[PAUSE_SHORT]".to_string());
            continue;
        }

        if clean.starts_with("- ") {
            lines.push(format!("• {}", clean.trim_start_matches("- ").trim()));
        } else {
            lines.push(clean);
        }
    }

    lines.join("\n")
}

fn strip_markdown_artifacts(input: &str) -> String {
    input
        .replace("**", "")
        .replace("__", "")
        .replace('`', "")
        .replace('#', "")
        .replace('>', "")
}

fn is_heading(line: &str) -> bool {
    matches!(line, "Direct Answer:" | "Evidence:" | "Citations:")
}
