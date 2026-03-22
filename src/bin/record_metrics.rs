//! bin/record_metrics.rs — Appends a session metrics row to METRICS.md.
//!
//! Run after each session to record: day, date, test results, file changes, lines.
//!
//! Usage:
//!   cargo run --bin record-metrics [-- --day N --date YYYY-MM-DD --from-sha SHA]

use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().collect::<Vec<_>>();
    let mut day = String::new();
    let mut date = String::new();
    let mut from_sha = String::new();

    for i in 1..args.len() {
        match args[i].as_str() {
            "--day" if i + 1 < args.len() => day = args[i + 1].clone(),
            "--date" if i + 1 < args.len() => date = args[i + 1].clone(),
            "--from-sha" if i + 1 < args.len() => from_sha = args[i + 1].clone(),
            _ => {}
        }
    }

    // DAY_COUNT format: "N YYYY-MM-DD" — first token is day number
    if day.is_empty() {
        day = std::fs::read_to_string("DAY_COUNT")
            .map(|c| parse_day_from_count(&c))
            .unwrap_or_else(|_| "?".to_string());
    }
    if date.is_empty() {
        date = std::process::Command::new("date")
            .arg("+%Y-%m-%d")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|_| "?".to_string());
    }

    // Run tests and capture results
    let test_output = Command::new("cargo")
        .args(["test", "--", "--nocapture"])
        .output()
        .ok();

    let (tests_passed, tests_failed) = if let Some(output) = &test_output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{stdout}\n{stderr}");
        let passed = count_test_result(&combined, "test result: ok");
        let failed = count_test_result(&combined, "test result: FAILED");
        (passed, failed)
    } else {
        (0, 0)
    };

    // Get git diff stats for the full session
    let (files_changed, lines_added, lines_removed) = get_git_diff_stats(&from_sha);

    // Build the metrics row
    let tokens_used = "N/A".to_string(); // Token usage requires API integration
    let committed = "yes".to_string();
    let notes = format!(
        "{} files, +{} lines, -{} lines",
        files_changed, lines_added, lines_removed
    );

    let row = format!(
        "| {day} | {date} | {tokens} | {passed} | {failed} | {files} | {added} | {removed} | {committed} | {notes} |",
        day = day,
        date = date,
        tokens = tokens_used,
        passed = tests_passed,
        failed = tests_failed,
        files = files_changed,
        added = lines_added,
        removed = lines_removed,
        committed = committed,
        notes = notes
    );

    // Append to METRICS.md, inserting before the marker comment
    let metrics_path = Path::new("METRICS.md");
    let content = match std::fs::read_to_string(metrics_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to read METRICS.md: {e}");
            return;
        }
    };

    match insert_before_marker(&content, &row) {
        Ok(new_content) => {
            if let Err(e) = std::fs::write(metrics_path, new_content) {
                eprintln!("Failed to write metrics: {e}");
            } else {
                println!("Recorded metrics: {}", row);
            }
        }
        Err(e) => {
            eprintln!("Failed to insert metrics row: {e}");
        }
    }
}

/// Insert a metrics row before the marker comment in METRICS.md content.
/// Returns the modified content, or an error if the marker is not found.
fn insert_before_marker(content: &str, row: &str) -> Result<String, &'static str> {
    let marker = "<!-- Sessions are appended below this line automatically -->";
    match content.find(marker) {
        Some(idx) => Ok(format!("{}{}\n{}", &content[..idx], row, &content[idx..])),
        None => Err("Marker not found"),
    }
}

fn count_test_result(output: &str, marker: &str) -> u32 {
    output
        .lines()
        .filter(|line| line.contains(marker))
        .filter_map(|line| {
            // Extract number from "N passed" or "N failed"
            let num_str = line.split_whitespace().find(|w| {
                w.chars()
                    .next()
                    .map(|c| c.is_ascii_digit())
                    .unwrap_or(false)
            })?;
            num_str.parse::<u32>().ok()
        })
        .sum()
}

fn get_git_diff_stats(from_sha: &str) -> (u32, u32, u32) {
    // If a session-start SHA was provided, diff the full session against it.
    // Otherwise fall back to diffing the last commit only.
    let from = if !from_sha.is_empty() {
        format!("{from_sha}..HEAD")
    } else {
        "HEAD^..HEAD".to_string()
    };

    let diff_output = Command::new("git")
        .args(["diff", &from, "--stat", "--stat-width=200"])
        .output()
        .ok();

    match diff_output {
        Some(o) if o.status.success() => {
            let output = String::from_utf8_lossy(&o.stdout);
            let last_line = output.lines().last().unwrap_or("");
            let parts: Vec<&str> = last_line.split_whitespace().collect();
            // Format: "N files changed, M insertions(+), L deletions(-)"
            // indices:  0    1       2    3      4          5      6
            let files = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
            let added = parts
                .get(3)
                .map(|s| s.replace(',', ""))
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            let removed = parts
                .get(5)
                .map(|s| s.replace(',', ""))
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            (files, added, removed)
        }
        _ => (0, 0, 0),
    }
}

/// Extract the day number from DAY_COUNT content (format: "N YYYY-MM-DD").
fn parse_day_from_count(content: &str) -> String {
    content
        .trim()
        .split_whitespace()
        .next()
        .unwrap_or("?")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_day_from_count_simple() {
        assert_eq!(parse_day_from_count("1"), "1");
    }

    #[test]
    fn test_parse_day_from_count_with_date() {
        assert_eq!(parse_day_from_count("1 2026-03-22"), "1");
    }

    #[test]
    fn test_parse_day_from_count_high_day() {
        assert_eq!(parse_day_from_count("42 2026-03-22"), "42");
    }

    #[test]
    fn test_parse_day_from_count_empty() {
        assert_eq!(parse_day_from_count(""), "?");
    }

    #[test]
    fn test_parse_day_from_count_whitespace() {
        assert_eq!(parse_day_from_count("   "), "?");
    }

    #[test]
    fn test_parse_day_from_count_extra_whitespace() {
        assert_eq!(parse_day_from_count("  5   2026-01-15  "), "5");
    }

    #[test]
    fn test_insert_before_marker_simple() {
        let marker = "<!-- Sessions are appended below this line automatically -->";
        let content = format!("header\n{}\nfooter", marker);
        let result = insert_before_marker(&content, "| row |");
        assert!(result.is_ok());
        let s = result.unwrap();
        assert!(s.contains(&format!("header\n| row |\n{}", marker)));
    }

    #[test]
    fn test_insert_before_marker_no_marker() {
        let content = "header\nfooter";
        let result = insert_before_marker(content, "| row |");
        assert!(result.is_err());
    }

    #[test]
    fn test_insert_before_marker_preserves_marker_line() {
        let marker = "<!-- Sessions are appended below this line automatically -->";
        let content = format!("header\n{}\nfooter", marker);
        let result = insert_before_marker(&content, "| row |").unwrap();
        assert!(result.contains(marker));
        assert!(result.starts_with("header"));
    }
}
