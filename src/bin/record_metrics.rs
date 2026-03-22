//! bin/record_metrics.rs — Appends a session metrics row to METRICS.md.
//!
//! Run after each session to record: day, date, test results, file changes, lines.
//!
//! Usage:
//!   cargo run --bin record-metrics [-- --day N --date YYYY-MM-DD]

use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().collect::<Vec<_>>();
    let mut day = String::new();
    let mut date = String::new();

    for i in 1..args.len() {
        match args[i].as_str() {
            "--day" if i + 1 < args.len() => day = args[i + 1].clone(),
            "--date" if i + 1 < args.len() => date = args[i + 1].clone(),
            _ => {}
        }
    }

    // Defaults
    if day.is_empty() {
        day = std::fs::read_to_string("DAY_COUNT")
            .map(|s| s.trim().to_string())
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

    // Get git diff stats from last commit
    let (files_changed, lines_added, lines_removed) = get_git_diff_stats();

    // Build the metrics row
    let tokens_used = "N/A".to_string(); // Token usage requires API integration
    let committed = "yes".to_string();
    let notes = format!(
        "{} files, +{} lines, -{} lines",
        files_changed, lines_added, lines_removed
    );

    let row = format!(
        "| {day} | {date} | {tokens} | {passed} | {failed} | {files} | {added} | {removed} | {committed} | {notes} |\n",
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

    // Append to METRICS.md
    let metrics_path = Path::new("METRICS.md");
    if let Ok(file) = OpenOptions::new().append(true).open(metrics_path) {
        let mut f = file;
        if let Err(e) = writeln!(f, "{}", row.trim()) {
            eprintln!("Failed to write metrics: {e}");
        } else {
            println!("Recorded metrics: {row}");
        }
    } else {
        eprintln!("Failed to open METRICS.md for appending");
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

fn get_git_diff_stats() -> (u32, u32, u32) {
    // Get stats from the last commit
    let output = Command::new("git")
        .args(["log", "-1", "--format=%H"])
        .output()
        .ok();

    let commit = match output {
        Some(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        _ => return (0, 0, 0),
    };

    // Get diff against parent
    let diff_output = Command::new("git")
        .args([
            "diff",
            &format!("{commit}^..{commit}"),
            "--stat",
            "--stat-width=200",
        ])
        .output()
        .ok();

    match diff_output {
        Some(o) if o.status.success() => {
            let output = String::from_utf8_lossy(&o.stdout);
            let last_line = output.lines().last().unwrap_or("");
            let parts: Vec<&str> = last_line.split_whitespace().collect();
            // Format: "N files changed, M insertions(+), L deletions(-)"
            let files = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
            let added = parts
                .get(2)
                .map(|s| s.replace(',', ""))
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            let removed = parts
                .get(4)
                .map(|s| s.replace(',', ""))
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            (files, added, removed)
        }
        _ => (0, 0, 0),
    }
}
