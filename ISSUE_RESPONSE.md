issue_number: 4
status: fixed
comment: Built check_caddyfile.rs — validates Caddyfile syntax and formatting with checks for mixed tabs/spaces indentation, balanced braces, and valid directive patterns. 21 tests covering valid blocks, directives, comments, and invalid patterns. Run with `cargo run --bin check_caddyfile -- <file>`.
