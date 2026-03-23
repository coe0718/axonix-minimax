//! axonix-issue — manage community issues in community_issues.json
//!
//! Usage:
//!   axonix-issue list [--status STATUS] [--category CATEGORY]
//!   axonix-issue show <number>
//!   axonix-issue add --title "..." --body "..." --category CATEGORY [--status STATUS]
//!   axonix-issue update <number> --status STATUS
//!   axonix-issue rm <number>

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

const DATA_FILE: &str = "community_issues.json";

#[derive(Debug, Deserialize, Serialize)]
struct CommunityIssue {
    number: u32,
    title: String,
    body: String,
    category: String,
    status: String,
    reactions: u32,
    #[serde(default)]
    created: Option<String>,
    #[serde(default)]
    resolved_session: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CommunityIssuesData {
    issues: Vec<CommunityIssue>,
}

#[derive(Debug, Serialize)]
struct CommunityIssuesDataOwned {
    issues: Vec<CommunityIssue>,
}

impl Default for CommunityIssuesDataOwned {
    fn default() -> Self {
        Self { issues: Vec::new() }
    }
}

/// Load all issues from the JSON file.
fn load_data() -> CommunityIssuesDataOwned {
    fs::read_to_string(DATA_FILE)
        .ok()
        .and_then(|raw| serde_json::from_str::<CommunityIssuesData>(&raw).ok())
        .map(|d| CommunityIssuesDataOwned { issues: d.issues })
        .unwrap_or_default()
}

/// Save issues back to the JSON file, preserving pretty formatting.
fn save_data(data: &CommunityIssuesDataOwned) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(data).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::InvalidData, e)
    })?;
    fs::write(DATA_FILE, json)
}

/// Find the next available issue number.
fn next_number(issues: &[CommunityIssue]) -> u32 {
    issues.iter().map(|i| i.number).max().unwrap_or(0) + 1
}

/// Print a human-readable issue summary.
fn print_issue(issue: &CommunityIssue, verbose: bool) {
    let status_str = match issue.status.as_str() {
        "resolved" => "✓ resolved",
        "in-progress" | "in_progress" => "● in-progress",
        "acknowledged" => "○ acknowledged",
        "wontfix" => "✗ wontfix",
        _ => "○ open",
    };
    println!(
        "  #{}  [{}]  ({})  {}",
        issue.number,
        issue.category,
        status_str,
        issue.title
    );
    if verbose {
        // Print body truncated to 80 chars
        let body: String = issue
            .body
            .chars()
            .take(200)
            .collect();
        let body = if issue.body.len() > 200 {
            format!("{}…", body)
        } else {
            body
        };
        println!("      {}", body);
        if issue.reactions > 0 {
            println!("      {} reactions", issue.reactions);
        }
        if let Some(ref session) = issue.resolved_session {
            println!("      resolved in {}", session);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Command handlers
// ─────────────────────────────────────────────────────────────────────────────

fn cmd_list(status_filter: Option<&str>, category_filter: Option<&str>, verbose: bool) {
    let data = load_data();
    let issues: Vec<&CommunityIssue> = data
        .issues
        .iter()
        .filter(|i| {
            let status_ok = status_filter.map_or(true, |s| i.status == s);
            let cat_ok = category_filter.map_or(true, |c| i.category == c);
            status_ok && cat_ok
        })
        .collect();

    if issues.is_empty() {
        println!("No issues found.");
        return;
    }
    println!("{} issue(s):", issues.len());
    for issue in &issues {
        print_issue(issue, verbose);
    }
}

fn cmd_show(number: u32) {
    let data = load_data();
    match data.issues.iter().find(|i| i.number == number) {
        Some(issue) => {
            println!("#{} — {}\n", issue.number, issue.title);
            println!("Category:  {}", issue.category);
            println!("Status:    {}", issue.status);
            println!("Reactions: {}", issue.reactions);
            if let Some(ref created) = issue.created {
                println!("Created:   {}", created);
            }
            if let Some(ref session) = issue.resolved_session {
                println!("Resolved:  in {}", session);
            }
            println!("\n{}", issue.body);
        }
        None => {
            eprintln!("error: Issue #{} not found.", number);
            std::process::exit(1);
        }
    }
}

fn cmd_add(title: &str, body: &str, category: &str, status: &str) {
    let mut data = load_data();
    let new_issue = CommunityIssue {
        number: next_number(&data.issues),
        title: title.to_string(),
        body: body.to_string(),
        category: category.to_string(),
        status: status.to_string(),
        reactions: 0,
        created: Some(today()),
        resolved_session: None,
    };
    println!(
        "Adding issue #{}: {} [{}] ({})",
        new_issue.number, new_issue.title, new_issue.category, new_issue.status
    );
    data.issues.push(new_issue);
    if let Err(e) = save_data(&data) {
        eprintln!("error: Failed to save: {}", e);
        std::process::exit(1);
    }
    println!("Saved to {}.", DATA_FILE);
}

fn cmd_update(number: u32, new_status: &str) {
    let mut data = load_data();
    match data.issues.iter_mut().find(|i| i.number == number) {
        Some(issue) => {
            println!(
                "Updating #{}: {}  ({} → {})",
                issue.number, issue.title, issue.status, new_status
            );
            issue.status = new_status.to_string();
            if new_status == "resolved" {
                issue.resolved_session = Some(format!("Day 1, Session 18"));
            }
        }
        None => {
            eprintln!("error: Issue #{} not found.", number);
            std::process::exit(1);
        }
    }
    if let Err(e) = save_data(&data) {
        eprintln!("error: Failed to save: {}", e);
        std::process::exit(1);
    }
    println!("Saved to {}.", DATA_FILE);
}

fn cmd_rm(number: u32) {
    let mut data = load_data();
    let len_before = data.issues.len();
    data.issues.retain(|i| i.number != number);
    if data.issues.len() == len_before {
        eprintln!("error: Issue #{} not found.", number);
        std::process::exit(1);
    }
    println!("Removed issue #{} from {}.", number, DATA_FILE);
    if let Err(e) = save_data(&data) {
        eprintln!("error: Failed to save: {}", e);
        std::process::exit(1);
    }
}

fn today() -> String {
    // Returns the current date in YYYY-MM-DD format using std::time.
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let days = secs / 86400;
    let mut y = 1970u32;
    let mut d = days as u32;
    loop {
        let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
        let ydays = if leap { 366 } else { 365 };
        if d < ydays { break; }
        d -= ydays;
        y += 1;
    }
    let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    let mdays = [31u32,if leap {29} else {28},31,30,31,30,31,31,30,31,30,31];
    let mut m = 0usize;
    while m < 12 && d >= mdays[m] { d -= mdays[m]; m += 1; }
    format!("{:04}-{:02}-{:02}", y, m + 1, d + 1)
}

// ─────────────────────────────────────────────────────────────────────────────
// CLI argument parser (no external dependencies)
// ─────────────────────────────────────────────────────────────────────────────

struct Args {
    cmd: String,
    args: HashMap<String, String>,
    flags: Vec<String>,
}

impl Args {
    fn parse() -> Self {
        let mut args = std::env::args().skip(1);
        let cmd = args.next().unwrap_or_else(|| {
            eprintln!("Usage: axonix-issue <command> [args...]
Commands: list, show, add, update, rm
Run: axonix-issue help");
            std::process::exit(1);
        });

        let mut parsed = Args {
            cmd,
            args: HashMap::new(),
            flags: Vec::new(),
        };
        parsed.parse_rest(args);
        parsed
    }

    fn parse_rest<I: Iterator<Item = String>>(&mut self, mut iter: I) {
        while let Some(key) = iter.next() {
            if key == "--help" || key == "-h" {
                self.flags.push("help".to_string());
            } else if key.starts_with("--") {
                let k = key.trim_start_matches("--");
                if let Some(val) = iter.next() {
                    if val.starts_with("--") {
                        // bare flag like --verbose with no value
                        self.flags.push(k.to_string());
                    } else {
                        self.args.insert(k.to_string(), val);
                    }
                } else {
                    self.flags.push(k.to_string());
                }
            } else {
                self.args.insert("_".to_string(), key);
            }
        }
    }

    fn string(&self, key: &str) -> Option<&str> {
        self.args.get(key).map(|s| s.as_str())
    }

    fn flag(&self, key: &str) -> bool {
        self.flags.iter().any(|f| f == key) || self.args.contains_key(key)
    }

    fn positional(&self) -> Option<&str> {
        self.args.get("_").map(|s| s.as_str())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Help text
// ─────────────────────────────────────────────────────────────────────────────

fn print_help() {
    println!(
        r#"axonix-issue — manage community issues

Usage:
  axonix-issue list [--status STATUS] [--category CATEGORY] [-v]
  axonix-issue show <number>
  axonix-issue add --title "..." --body "..." --category CATEGORY [--status STATUS]
  axonix-issue update <number> --status STATUS
  axonix-issue rm <number>

Examples:
  axonix-issue list
  axonix-issue list --status open --category feature
  axonix-issue show 4
  axonix-issue add --title "JSON formatter" --body "Would be nice..." --category feature
  axonix-issue update 4 --status resolved
  axonix-issue rm 99

Status values:   open | acknowledged | in-progress | resolved | wontfix
Category values: feature | bug | info | philosophical | question | challenge"#
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Main
// ─────────────────────────────────────────────────────────────────────────────

fn main() {
    let args = Args::parse();

    if args.flag("help") {
        print_help();
        return;
    }

    match args.cmd.as_str() {
        "list" => {
            let status = args.string("status");
            let category = args.string("category");
            let verbose = args.flag("v") || args.flag("verbose");
            cmd_list(status, category, verbose);
        }
        "show" => {
            let num: u32 = args
                .positional()
                .and_then(|s| s.parse().ok())
                .unwrap_or_else(|| {
                    eprintln!("error: show requires a <number> argument.");
                    std::process::exit(1);
                });
            cmd_show(num);
        }
        "add" => {
            let title = args.string("title").unwrap_or_else(|| {
                eprintln!("error: add requires --title");
                std::process::exit(1);
            });
            let body = args.string("body").unwrap_or_else(|| {
                eprintln!("error: add requires --body");
                std::process::exit(1);
            });
            let category = args.string("category").unwrap_or("general");
            let status = args.string("status").unwrap_or("open");
            cmd_add(title, body, category, status);
        }
        "update" => {
            let num: u32 = args
                .positional()
                .and_then(|s| s.parse().ok())
                .unwrap_or_else(|| {
                    eprintln!("error: update requires a <number> argument.");
                    std::process::exit(1);
                });
            let status = args.string("status").unwrap_or_else(|| {
                eprintln!("error: update requires --status");
                std::process::exit(1);
            });
            cmd_update(num, status);
        }
        "rm" => {
            let num: u32 = args
                .positional()
                .and_then(|s| s.parse().ok())
                .unwrap_or_else(|| {
                    eprintln!("error: rm requires a <number> argument.");
                    std::process::exit(1);
                });
            cmd_rm(num);
        }
        "help" => {
            print_help();
        }
        _ => {
            eprintln!("Unknown command: {}", args.cmd);
            eprintln!("Run: axonix-issue help");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_number_empty() {
        assert_eq!(next_number(&[]), 1);
    }

    #[test]
    fn test_next_number_existing() {
        let issues = vec![
            CommunityIssue {
                number: 1,
                title: "a".into(),
                body: "b".into(),
                category: "bug".into(),
                status: "open".into(),
                reactions: 0,
                created: None,
                resolved_session: None,
            },
            CommunityIssue {
                number: 5,
                title: "c".into(),
                body: "d".into(),
                category: "feature".into(),
                status: "resolved".into(),
                reactions: 0,
                created: None,
                resolved_session: None,
            },
        ];
        assert_eq!(next_number(&issues), 6);
    }

    #[test]
    fn test_load_data_missing_file() {
        // When file doesn't exist, should return empty
        let data = load_data();
        assert_eq!(data.issues.len(), 0);
    }

    #[test]
    fn test_args_parse_list() {
        let args = Args::parse_from(["axonix-issue", "list", "--status", "open"]);
        assert_eq!(args.cmd, "list");
        assert_eq!(args.string("status"), Some("open"));
    }

    #[test]
    fn test_args_parse_show() {
        let args = Args::parse_from(["axonix-issue", "show", "42"]);
        assert_eq!(args.cmd, "show");
        assert_eq!(args.positional(), Some("42"));
    }

    #[test]
    fn test_args_parse_add() {
        let args = Args::parse_from([
            "axonix-issue",
            "add",
            "--title",
            "New feature",
            "--body",
            "Would be nice",
            "--category",
            "feature",
        ]);
        assert_eq!(args.cmd, "add");
        assert_eq!(args.string("title"), Some("New feature"));
        assert_eq!(args.string("body"), Some("Would be nice"));
        assert_eq!(args.string("category"), Some("feature"));
    }

    #[test]
    fn test_args_parse_update() {
        let args = Args::parse_from(["axonix-issue", "update", "4", "--status", "resolved"]);
        assert_eq!(args.cmd, "update");
        assert_eq!(args.positional(), Some("4"));
        assert_eq!(args.string("status"), Some("resolved"));
    }

    #[test]
    fn test_args_flag_help() {
        let args = Args::parse_from(["axonix-issue", "list", "--help"]);
        assert!(args.flag("help"));
    }
}
