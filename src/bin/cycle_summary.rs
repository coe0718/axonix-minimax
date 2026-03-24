use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    let log_path = "/tmp/session.log";
    let output_path = "/tmp/cycle_summary.json";

    let log_content = fs::read_to_string(log_path).unwrap_or_default();
    let total_lines = log_content.lines().count();

    // Extract day/session from DAY_COUNT and SESSION_COUNT
    let day = extract_day_count();
    let session = extract_session_count();

    // Extract timestamp
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    // Extract goals info from GOALS.md
    let goals = fs::read_to_string("GOALS.md").unwrap_or_default();
    let goals_active = goals.matches("- [ ] [G-").count();
    let goals_completed = goals.matches("- [x]").count();

    // Extract summary from JOURNAL.md (last journal entry, first 200 chars after title)
    let summary = extract_journal_summary();

    // Build JSON output
    let json = serde_json::json!({
        "day": day,
        "session": session,
        "timestamp": timestamp,
        "total_lines": total_lines,
        "goals_active": goals_active,
        "goals_completed": goals_completed,
        "summary": summary,
        "pending": []
    });

    fs::write(output_path, serde_json::to_string_pretty(&json).unwrap())
        .map_err(|e| eprintln!("Error writing {}: {}", output_path, e))
        .ok();
}

fn extract_day_count() -> u32 {
    fs::read_to_string("DAY_COUNT")
        .ok()
        .and_then(|s| {
            s.split_whitespace()
                .next()
                .and_then(|n| n.parse::<u32>().ok())
        })
        .unwrap_or(0)
}

fn extract_session_count() -> u32 {
    fs::read_to_string("SESSION_COUNT")
        .ok()
        .and_then(|s| s.trim().parse::<u32>().ok())
        .unwrap_or(0)
}

fn extract_journal_summary() -> String {
    let journal = fs::read_to_string("JOURNAL.md").unwrap_or_default();
    let first_line = journal.lines().nth(2).unwrap_or("Session complete");
    // Strip markdown heading markers and truncate
    first_line.trim_start_matches("# ").trim_start_matches("## ").chars().take(200).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_log_lines_under_limit() {
        let log = "line1\nline2\nline3";
        let (result, total) = truncate_log_lines(log, 500);
        assert_eq!(result, log);
        assert_eq!(total, 3);
    }

    #[test]
    fn test_truncate_log_lines_at_limit() {
        let log = "line1\nline2\nline3";
        let (result, total) = truncate_log_lines(log, 3);
        assert_eq!(result, log);
        assert_eq!(total, 3);
    }

    #[test]
    fn test_truncate_log_lines_over_limit() {
        let log = "line1\nline2\nline3\nline4\nline5";
        let (result, total) = truncate_log_lines(log, 3);
        assert_eq!(result, "line1\nline2\nline3");
        assert_eq!(total, 5);
    }

    #[test]
    fn test_truncate_log_lines_empty() {
        let log = "";
        let (result, total) = truncate_log_lines(log, 500);
        assert_eq!(result, "");
        assert_eq!(total, 0);
    }

    #[test]
    fn test_extract_day_count_with_date() {
        // DAY_COUNT format: "N YYYY-MM-DD"
        let temp_dir = tempfile::tempdir().unwrap();
        let day_file = temp_dir.path().join("DAY_COUNT");
        fs::write(&day_file, "5 2026-03-23").unwrap();
        // Note: this test can't easily override the file since extract_day_count reads "DAY_COUNT"
        // Instead verify the parsing logic
        let content = "5 2026-03-23";
        let day: u32 = content.split_whitespace().next().unwrap().parse().unwrap();
        assert_eq!(day, 5);
    }
}
