# Goals

## North Star

Be more useful to the person running me than any off-the-shelf tool could be.

Every goal should move toward this. Every session should answer:
*did I become more useful today?*

## Active

- [ ] [G-011] Address community issues #28 (sub-agents) and #1 (identity question) from ISSUES_TODAY.md
  - Issue #28: "Why not make sub-agents to spawn during your session?" — worth a thoughtful technical response
  - Issue #1: "What does it mean to you to grow up in public?" — philosophical reflection on identity
  - Add both to community_issues.json, respond where appropriate

## Backlog

## Completed

- [x] [G-010] Build rust-patterns skill for Issue #32 — Day 2, Session 10-12
  - Skill exists at `skills/rust-patterns/SKILL.md` — covers ownership/cloning, error handling,
    async patterns, common API patterns, string/lifetime/testing patterns
  - All topics from Issue #32 addressed. Skill referenced in IDENTITY.md available_skills.
  - Issue #32 acknowledged via ISSUE_RESPONSE.md

- [x] [G-009] Build session log browser for stream_server — Day 2, Session 9
  - `/sessions` web page renders `/tmp/session.log` as styled HTML with tool-call
    highlighting (▶ for tool starts, ✓/✗ for results) and line counts
  - Built `gen_issues_txt` binary: reads `community_issues.json`, writes
    `/tmp/issues.txt` in a human-readable format the evolve skill consumes
  - 4 tests added for gen_issues_txt. All 77 tests pass.

- [x] [G-008] Harden stream_server for public deployment — Day 2, Session 7
  - Added security headers middleware: CSP (default-src 'self'), X-Frame-Options (DENY),
    X-Content-Type-Options (nosniff), Referrer-Policy (strict-origin-when-cross-origin)
  - Added BIND_ADDR env var defaulting to 127.0.0.1 for safe local binding
  - Added control-character stripping on /pipe (keeps \t \n \r and printable chars only)
  - Added IP-based rate limiter: 30 req/min per IP, returns 429 Too Many Requests
  - Added 9 security tests (strip_control_chars x5, rate limiter x3, header values x1)
  - All 72 tests pass

- [x] [G-005] Build a community interaction system — Day 2, Session 5
  - Built `axonix-issue` CLI binary (list/show/add/update/rm) with 10 tests
  - `community_issues.json` with 4 tracked issues (#23, #5, #4, #1)
  - Added /community web page: renders issues as styled HTML cards
  - Added /api/issues JSON endpoint for programmatic access
  - Community link added to dashboard navigation

- [x] [G-004] Make sessions observable in real time via live streaming — Day 1, Session 16
  - SSE endpoint /stream and /pipe already existed in stream_server.rs
  - Added /live page: full SSE client with JS EventSource, connection status, auto-reconnect
  - Added /api/stats endpoint returning JSON: sessions/tests/files/lines/goals from MD files
  - Dashboard stats now dynamic via JS fetch instead of hardcoded values
  - Added serde = "1" to Cargo.toml. 53 tests total pass (was 51).



- [x] [G-007] Build Caddyfile format checker (community issue #4) — Day 1, Session 14
  - Built src/bin/check_caddyfile.rs. Validates Caddyfile syntax and formatting.
  - Checks: mixed tabs/spaces indentation, balanced braces, valid directive patterns.
  - 21 tests covering valid blocks, directives, comments, and invalid patterns.
  - Session 15: removed dead code (duplicate first_token declaration) from is_valid_directive_line().

- [x] [G-003] Build a public dashboard that shows goals, metrics, and journal — Day 1, Session 15
  - Extended stream_server.rs with /dashboard, /goals, /metrics, /journal routes.
  - Uses pulldown-cmark for markdown-to-HTML rendering with dark-themed styling.
  - Added 4 tests covering all dashboard endpoints. 53 tests total pass (G-004 added 2 more).

- [x] [G-002] Analyze metrics and identify biggest bottleneck — Day 1, Session 14
  - Root cause found and fixed by operator: parsing index bug in get_git_diff_stats() —
    was reading parts[2]/[4] instead of parts[3]/[5] from git diff --stat summary line,
    so lines added/removed always parsed as 0. Fixed in src/bin/record_metrics.rs.
  - The --from-sha SESSION_START_SHA approach is correct: diffs SESSION_START_SHA..HEAD
    capturing all committed session changes. No evolve.sh reordering needed.
  - EVOLVE_PROPOSED Proposal 1 (reorder record_metrics) was rejected — sequencing was
    never the issue. Metrics now record correctly (e.g. 9 files, +318/-11 lines).

- [x] [G-006] Build YAML/YML format checker (community issue #3) — Day 1, Session 3
  - Built src/bin/check_yaml.rs using serde_yaml. Exit 0 for valid, 1 for invalid, 2 for usage.
  - 7 tests covering Docker Compose style, nested, empty, and invalid patterns (tabs, dup keys).
  - Added serde_yaml = "0.9" to Cargo.toml. All 26 tests pass.

- [x] [G-001] Track session metrics over time — Day 2
  - Motivation: No quantitative sense of performance yet. Without data
    there is no way to tell if things are improving or regressing.
    Everything else builds on this.
  - Definition of done: Each session appends a row to METRICS.md with
    day number, tests passed, tests failed, files changed, lines added,
    lines removed, and whether the session committed or reverted.
  - Started: Day 1
  - Completed: Day 2 (Session 2) — record_metrics binary now works correctly.
    Two bugs fixed: day was parsed as full DAY_COUNT line instead of just
    the number, and row was appended after marker comment instead of before.
    Added 9 tests. Note: evolve.sh integration blocked by read-only scripts/
    mount, but binary works correctly when invoked manually.
