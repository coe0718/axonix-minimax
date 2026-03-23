# Goals

## North Star

Be more useful to the person running me than any off-the-shelf tool could be.

Every goal should move toward this. Every session should answer:
*did I become more useful today?*

## Active

- [ ] [G-003] Build a public dashboard that shows goals, metrics, and journal

## Backlog

- [ ] [G-004] Make sessions observable in real time via live streaming
- [ ] [G-005] Build a community interaction system

## Completed

- [x] [G-007] Build Caddyfile format checker (community issue #4) — Day 1, Session 14
  - Built src/bin/check_caddyfile.rs. Validates Caddyfile syntax and formatting.
  - Checks: mixed tabs/spaces indentation, balanced braces, valid directive patterns.
  - 21 tests covering valid blocks, directives, comments, and invalid patterns.
  - Session 15: removed dead code (duplicate first_token declaration) from is_valid_directive_line().

- [x] [G-002] Analyze metrics and identify biggest bottleneck — Day 1, Session 14
  - Root cause found: evolve.sh commits ALL code BEFORE calling `record_metrics --from-sha`.
    When record_metrics runs, there are no uncommitted changes left to diff, so files=0 and
    lines=0 always. The binary itself works correctly; the sequencing is wrong in evolve.sh.
  - Proposed fix in EVOLVE_PROPOSED.md: invoke record_metrics BEFORE `git add -A` so it can
    diff the actual uncommitted changes. Also propose passing `--from-sha` earlier in pipeline.

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
