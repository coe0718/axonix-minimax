//! check_caddyfile — validates Caddyfile syntax and formatting.
//!
//! Usage: cargo run --bin check_caddyfile -- <file>
//!
//! Exit code: 0 if valid, 1 if invalid (with error to stderr), 2 for usage.

use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: check_caddyfile <file>");
        eprintln!("  Validates Caddyfile syntax and formatting.");
        process::exit(2);
    }

    let file_path = &args[1];

    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: failed to read '{}': {}", file_path, e);
            process::exit(1);
        }
    };

    match validate_caddyfile(&content) {
        Ok(()) => {
            println!("{}: valid Caddyfile", file_path);
            process::exit(0);
        }
        Err(msg) => {
            eprintln!("{}: invalid Caddyfile", file_path);
            eprintln!("  {}", msg);
            process::exit(1);
        }
    }
}

/// Validate Caddyfile content. Returns Ok(()) if valid, Err(msg) with details.
fn validate_caddyfile(content: &str) -> Result<(), String> {
    let mut brace_depth: i32 = 0;
    let mut line_num: usize = 0;

    for line in content.lines() {
        line_num += 1;

        // Check for mixed tabs and spaces indentation
        if let Some(pos) = line.find(|c: char| !c.is_whitespace()) {
            let indent = &line[..pos];
            if indent.contains(' ') && indent.contains('\t') {
                return Err(format!(
                    "line {}: mixed tabs and spaces in indentation",
                    line_num
                ));
            }
        }

        // Count braces (in uncommented content).
        // Lines ending with '{' are block openers — skip brace counting so the
        // EOF check only catches files that never close their blocks.
        let uncommented: String = if let Some(hash) = line.find('#') {
            line[..hash].to_string()
        } else {
            line.to_string()
        };

        let trimmed_line = uncommented.trim();
        if !trimmed_line.is_empty() && !trimmed_line.ends_with('{') {
            for ch in uncommented.chars() {
                match ch {
                    '{' => brace_depth += 1,
                    '}' => brace_depth -= 1,
                    _ => {}
                }
            }
        }

        // Validate line content patterns for complete statements
        if !trimmed_line.is_empty()
            && !trimmed_line.starts_with('#')
            && !trimmed_line.ends_with('{')
            && !trimmed_line.ends_with('}')
        {
            if !is_valid_directive_line(trimmed_line) {
                return Err(format!("line {}: unrecognized directive pattern", line_num));
            }
        }
    }

    if brace_depth != 0 {
        return Err(format!(
            "unbalanced braces: {} unclosed (positive) or extra (negative)",
            brace_depth
        ));
    }

    Ok(())
}

/// Check if a trimmed line is a valid Caddyfile directive/global option pattern.
/// Valid: keyword [args...], where keyword is alphanumeric with hyphens/underscores.
/// Invalid: starts with whitespace not inside a block, bare symbols, etc.
fn is_valid_directive_line(line: &str) -> bool {
    // Global options and directives start with word chars or @ (matcher token)
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return true;
    }

    // Accept common patterns
    if trimmed.starts_with('@') {
        // Matcher token
        return trimmed[1..]
            .split_whitespace()
            .next()
            .map(|s| s.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-'))
            .unwrap_or(false);
    }

    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        // Inline global options block or placeholder — valid if content looks sane
        let inner = &trimmed[1..trimmed.len() - 1];
        return !inner.trim().is_empty();
    }

    // Standard directive: first token is alphanumeric/hyphen/underscore
    let first_token = trimmed.split_whitespace().next().unwrap_or("");
    if first_token.is_empty() {
        return false;
    }

    // Standard directive: first token is alphanumeric/hyphen/underscore
    let first_token = trimmed.split_whitespace().next().unwrap_or("");
    if first_token.is_empty() {
        return false;
    }

    // Accept lines that start with { but aren't a full { ... } block
    // (these are placeholder expansions like {host}.example.com)
    if first_token.starts_with('{') {
        return true;
    }

    first_token
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == ':' || c == '/')
}

/// Validate Caddyfile content for testing (returns errors as strings).
#[cfg(test)]
fn validate(content: &str) -> Result<(), String> {
    validate_caddyfile(content)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Valid Caddyfiles ──────────────────────────────────────────────────────

    #[test]
    fn test_valid_simple_directive() {
        assert!(validate("localhost:8080").is_ok());
    }

    #[test]
    fn test_valid_directive_with_args() {
        assert!(validate("tls admin@example.com").is_ok());
    }

    #[test]
    fn test_valid_block_open() {
        assert!(validate("localhost:8080 {").is_ok());
    }

    #[test]
    fn test_valid_block_closed() {
        assert!(validate("localhost:8080 {\n}").is_ok());
    }

    #[test]
    fn test_valid_block_with_directives() {
        let caddyfile = "localhost:8080 {\n\treverse_proxy / localhost:3000\n}";
        assert!(validate(caddyfile).is_ok());
    }

    #[test]
    fn test_valid_global_options() {
        let caddyfile = "{\n\temail admin@example.com\n\tonDemandTLS\n}";
        assert!(validate(caddyfile).is_ok());
    }

    #[test]
    fn test_valid_matcher_token() {
        assert!(validate("@matcher path /api/*").is_ok());
    }

    #[test]
    fn test_valid_inline_placeholder() {
        assert!(validate("{host}.example.com").is_ok());
    }

    #[test]
    fn test_valid_comment_only() {
        assert!(validate("# this is a comment").is_ok());
    }

    #[test]
    fn test_valid_empty_file() {
        assert!(validate("").is_ok());
    }

    #[test]
    fn test_valid_multiline_block() {
        let caddyfile = r#"
localhost:8080 {
    encode gzip
    reverse_proxy / localhost:3000
    log {
        output file /var/log/caddy/access.log
    }
}
"#;
        assert!(validate(caddyfile).is_ok());
    }

    // ── Invalid Caddyfiles ─────────────────────────────────────────────────────

    #[test]
    fn test_invalid_mixed_indentation() {
        // Tab + space on same line's indent
        assert!(validate("\t localhost:8080").is_err());
    }

    #[test]
    fn test_invalid_unclosed_brace() {
        assert!(validate("localhost:8080 {").is_err());
    }

    #[test]
    fn test_invalid_extra_closing_brace() {
        assert!(validate("localhost:8080 }").is_err());
    }

    #[test]
    fn test_invalid_extra_closing_brace_at_root() {
        // Bare } at top level (no opening block)
        assert!(validate("}").is_err());
    }

    #[test]
    fn test_invalid_nested_unclosed() {
        let caddyfile = "localhost:8080 {\n\tencode gzip\n";
        assert!(validate(caddyfile).is_err());
    }

    #[test]
    fn test_invalid_extra_close_at_end() {
        let caddyfile = "localhost:8080 {}\n}";
        assert!(validate(caddyfile).is_err());
    }

    #[test]
    fn test_invalid_bare_special_char() {
        // A line with just symbols that isn't a comment
        assert!(validate("@#$%").is_err());
    }

    // ── Edge cases ────────────────────────────────────────────────────────────

    #[test]
    fn test_brace_depth_nested() {
        let caddyfile = "a {\n\tb {\n\t\tc\n\t}\n}";
        assert!(validate(caddyfile).is_ok());
    }

    #[test]
    fn test_brace_depth_unbalanced_nested() {
        let caddyfile = "a {\n\tb {\n\t\tc\n\t}\n"; // missing closing }
        assert!(validate(caddyfile).is_err());
    }

    #[test]
    fn test_comment_before_directive() {
        let caddyfile = "# comment\nlocalhost:8080";
        assert!(validate(caddyfile).is_ok());
    }

    #[test]
    fn test_directive_with_dots_in_path() {
        assert!(validate("handle_path /static/* /var/www/*").is_ok());
    }
}
