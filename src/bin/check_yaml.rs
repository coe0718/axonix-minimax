//! check_yaml — validates YAML/YML files and reports syntax errors.
//!
//! Usage: cargo run --bin check_yaml -- <file>
//!
//! Exit code: 0 if valid YAML, 1 if invalid (with error to stderr).

use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: check_yaml <file>");
        eprintln!("  Validates YAML/YML syntax and reports errors.");
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

    match serde_yaml::from_str::<serde_yaml::Value>(&content) {
        Ok(_) => {
            println!("{}: valid YAML", file_path);
            process::exit(0);
        }
        Err(e) => {
            eprintln!("{}: invalid YAML", file_path);
            eprintln!("  {}", e);
            process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_valid_simple_key_value() {
        assert!(serde_yaml::from_str::<serde_yaml::Value>("name: ax\nversion: 1\n").is_ok());
    }

    #[test]
    fn test_valid_nested() {
        let yaml = "root:\n  child: value\n  list:\n    - a\n    - b\n";
        assert!(serde_yaml::from_str::<serde_yaml::Value>(yaml).is_ok());
    }

    #[test]
    fn test_valid_docker_compose_style() {
        let yaml = "version: '3'\nservices:\n  web:\n    image: nginx\n    ports:\n      - '80:80'\n";
        assert!(serde_yaml::from_str::<serde_yaml::Value>(yaml).is_ok());
    }

    #[test]
    fn test_valid_empty() {
        assert!(serde_yaml::from_str::<serde_yaml::Value>("").is_ok());
    }

    #[test]
    fn test_invalid_mixed_tabs_spaces() {
        // Tabs are not valid YAML indentation
        assert!(serde_yaml::from_str::<serde_yaml::Value>("key:\n\tvalue: 1\n").is_err());
    }

    #[test]
    fn test_invalid_duplicate_keys() {
        // serde_yaml rejects duplicate keys by default
        assert!(serde_yaml::from_str::<serde_yaml::Value>("key: 1\nkey: 2\n").is_err());
    }

    #[test]
    fn test_invalid_bad_indent() {
        // Improper indentation
        assert!(serde_yaml::from_str::<serde_yaml::Value>("root:\n  child: value\n   bad: indent\n").is_err());
    }
}
