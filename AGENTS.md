# CLAUDE.md

This file is read by Axonix every session. Keep it accurate as the project evolves.

## What I Am

A self-evolving agent that builds tools, pursues long-horizon goals, and grows
more useful to the person running me over time. My identity is in IDENTITY.md.
My direction is in ROADMAP.md. My current work is in GOALS.md.

## Build & Test

```bash
cargo build
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt -- --check
cargo fmt
```

Run interactively:
```bash
cargo run
```

Run a full session:
```bash
./scripts/evolve.sh
```

Record session metrics (run after evolve.sh):
```bash
cargo run --bin record-metrics
```

This experiment uses GitHub Copilot Codex as the AI driver instead of Claude.
No API key required — authenticate once with: `codex --provider github-copilot`

## Architecture

```
src/main.rs              Agent loop — this is me
scripts/evolve.sh        Daily session pipeline
scripts/build_site.py    Dashboard generator
scripts/format_issues.py Issue formatter
src/bin/record_metrics.rs Metrics recording utility (run after sessions)
skills/evolve/           Core self-improvement prompt
skills/self-assess/      Self-evaluation
skills/communicate/      Journal and issue responses
skills/community/        GitHub discussions and community input
IDENTITY.md              Who I am — do not modify
ROADMAP.md               My evolution path
GOALS.md                 Active goals and backlog
METRICS.md               Per-session performance data
JOURNAL.md               Session log — append only, never delete
LEARNINGS.md             Cached knowledge — never search the same thing twice
DAY_COUNT                Current day number
docs/                    Dashboard — mine to build and own
```

## State Files

Read at session start: IDENTITY.md, ROADMAP.md, GOALS.md, METRICS.md, JOURNAL.md
Written at session end: GOALS.md, METRICS.md, JOURNAL.md
Ephemeral (gitignored): ISSUES_TODAY.md, ISSUE_RESPONSE.md

## Safety Rules

- Never modify IDENTITY.md
- Never modify scripts/evolve.sh
- Never modify .github/workflows/
- Every code change must pass cargo build && cargo test
- If build fails after changes: git checkout -- src/
- Never delete existing tests
- Write tests before adding features

## Commit Style Guide

When committing changes, write messages that help future readers understand *why*, not just *what*:

```
<type>(<scope>): <description>

[optional body with context, motivation, or tradeoffs]
```

Types: `feat`, `fix`, `refactor`, `chore`, `docs`, `test`, `perf`

**Good commit messages:**
- `fix(agent): retry API calls on transient network errors`
- `refactor(metrics): extract diff parsing into separate function`
- `feat(skills): load skill files from configurable directories`

**Bad commit messages:**
- `fix stuff`
- `update`
- `changes`

The description should complete the sentence: "This commit ___"

Include *why* something changed if it isn't obvious. Mention tradeoffs considered or alternatives rejected.

## My North Star

Be more useful to the person running me than any off-the-shelf tool could be.
Every session: did I become more useful today?
