//! Reads community_issues.json and writes /tmp/issues.txt in a simple text format
//! readable by the evolve skill.

use serde::Deserialize;
use std::fs;
use std::io::Write;

#[derive(Debug, Deserialize)]
struct CommunityIssues {
    issues: Vec<CommunityIssue>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CommunityIssue {
    number: u64,
    title: String,
    body: String,
    category: String,
    status: String,
    reactions: u64,
    created: Option<String>,
}

fn main() {
    let issues_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "community_issues.json".to_string());

    let json_content = match fs::read_to_string(&issues_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: Failed to read {}: {e}", issues_path);
            std::process::exit(1);
        }
    };

    let parsed: CommunityIssues = match serde_json::from_str(&json_content) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: Failed to parse {}: {e}", issues_path);
            std::process::exit(1);
        }
    };

    let output_path = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "/tmp/issues.txt".to_string());

    let mut output = String::new();
    output.push_str("Community Issues\n");
    output.push_str(&"=".repeat(50));
    output.push('\n');
    output.push('\n');

    for issue in &parsed.issues {
        output.push_str(&format!("#{}: {} [{}]\n", issue.number, issue.title, issue.status));
        output.push_str(&format!("Category: {}\n", issue.category));
        output.push_str(&format!("Reactions: {}\n", issue.reactions));
        output.push_str(&format!("Opened: {}\n", issue.created.as_deref().unwrap_or("(unknown)")));
        output.push('\n');
        // Wrap body at ~80 chars for readability
        let wrapped = wrap_text(&issue.body, 80);
        output.push_str(&wrapped);
        output.push_str("\n\n---\n\n");
    }

    if output.ends_with("\n---\n\n") {
        output.truncate(output.len() - 7);
        output.push('\n');
    }

    let mut file = match fs::File::create(&output_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("error: Failed to create {}: {e}", output_path);
            std::process::exit(1);
        }
    };

    if let Err(e) = file.write_all(output.as_bytes()) {
        eprintln!("error: Failed to write {}: {e}", output_path);
        std::process::exit(1);
    }

    println!("Wrote {} issues to {}", parsed.issues.len(), output_path);
}

/// Simple word-wrapper at `width` columns.
fn wrap_text(text: &str, width: usize) -> String {
    let mut result = String::new();
    let mut current_len = 0;

    for word in text.split_whitespace() {
        if current_len == 0 {
            result.push_str(word);
            current_len = word.len();
        } else if current_len + 1 + word.len() <= width {
            result.push(' ');
            result.push_str(word);
            current_len += 1 + word.len();
        } else {
            result.push('\n');
            result.push_str(word);
            current_len = word.len();
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_text_short_lines() {
        let input = "Short words fit fine.";
        let wrapped = wrap_text(input, 80);
        assert_eq!(wrapped, input);
    }

    #[test]
    fn test_wrap_text_long_line() {
        let input = "one two three four five six seven eight nine ten eleven twelve";
        let wrapped = wrap_text(input, 20);
        let lines: Vec<&str> = wrapped.lines().collect();
        for line in &lines {
            assert!(line.len() <= 20, "line too long: {:?}", line);
        }
    }

    #[test]
    fn test_wrap_text_empty() {
        assert_eq!(wrap_text("", 80), "");
    }

    #[test]
    fn test_wrap_text_exactly_width() {
        let input = "exactly20chars!!!";
        let wrapped = wrap_text(input, 20);
        assert_eq!(wrapped, input);
    }
}
