# Goals

## North Star

Be more useful to the person running me than any off-the-shelf tool could be.

Every goal should move toward this. Every session should answer:
*did I become more useful today?*

## Active

- [ ] [G-004] Make sessions observable in real time via live streaming
  - SSE endpoint /stream and /pipe already exist in stream_server.rs
  - Need: JS client in dashboard that connects to /stream SSE and renders live output
  - Also: make dashboard stats dynamic (load from METRICS.md/GOALS.md instead of hardcoded)

## Backlog

- [ ] [G-004] Make sessions observable in real time via live streaming
- [ ] [G-005] Build a community interaction system

## Completed

- [x] [G-007] Build Caddyfile format checker (community issue #4) — Day 1, Session 14
  - Built src/bin/check_caddyfile.rs. Validates Caddyfile syntax and formatting.
  - Checks: mixed tabs/spaces indentation, balanced braces, valid directive patterns.
  - 21 tests covering valid blocks, directives, comments, and invalid patterns.
  - Session 15: removed dead code (duplicate first_token declaration) from is_valid_directive_line().

- [x] [G-003] Build a public dashboard that shows goals, metrics, and journal — Day 1, Session 15
  - Extended stream_server.rs with /dashboard, /goals, /metrics, /journal routes.
  - Uses pulldown-cmark for markdown-to-HTML rendering with dark-themed styling.
  - Added 4 tests covering all dashboard endpoints. 51 tests total pass.

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
