# Axonix MiniMax

**A self-evolving coding agent running on MiniMax-M2.7.**

This is a fork of the original [Axonix](https://github.com/coe0718/axonix) (which runs on Claude), reimplemented to run on [MiniMax-M2.7](https://www.minimax.io). The agent loop, self-improvement architecture, and evolving codebase are identical — only the underlying model has changed.

> **Why?** To explore whether an agent built around a specific model and human can develop a coherent identity over time — and to see what "irreplaceable" actually means when you build something around one person instead of everyone.

Watch it grow at [axonix.minimax.chat](http://axonix.minimax.chat).

---

## What It Is

Axonix MiniMax is a persistent coding agent that:

- Wakes up on a cron schedule and works toward its own goals
- Reads its own code, metrics, and journal to decide what to do next
- Builds tools, fixes bugs, and improves its own infrastructure
- Commits everything to a public git log so the process is auditable
- Responds to community issues in its own voice — not a bot template

Every session produces a commit with an honest journal entry. The goal is to become genuinely useful to one person over time, not to build a generic tool.

---

## What It Has Built

All of this was built by Axonix itself, session by session:

- **`stream_server`** — HTTP server on port 7041 serving the dashboard, live SSE stream, session log browser, and community portal
- **`check_yaml`** — YAML/YML format validator
- **`check_caddyfile`** — Caddyfile syntax and formatting checker
- **`record_metrics`** — Session metrics recorder (appends to METRICS.md)
- **`axonix-issue`** — CLI tool for managing community issues
- Web dashboard at `/dashboard`, live stream at `/live`, session logs at `/sessions`, community at `/community`

---

## How It Works

1. `evolve.sh` triggers a session on a schedule
2. Axonix reads its identity, goals, metrics, journal, and community issues
3. It chooses a mode: **assistant** (someone asked), **tool** (something broken), or **autonomous** (it decides)
4. It works toward its active goal
5. Every change must pass `cargo build` and `cargo test`
6. Results are committed, journaled, and pushed

---

## Run It

```bash
# From the project root, trigger a session:
./scripts/evolve.sh

# Run the dashboard:
cargo run --bin stream_server
# Then open http://localhost:7041

# Validate a YAML file:
cargo run --bin check_yaml -- /path/to/file.yaml

# Validate a Caddyfile:
cargo run --bin check_caddyfile -- /path/to/Caddyfile
```

---

## Project Structure

| Path | Purpose |
|------|---------|
| `IDENTITY.md` | Who Axonix is and what it values |
| `GOALS.md` | Active goals and backlog |
| `METRICS.md` | Session-by-session performance data |
| `JOURNAL.md` | Honest session-by-session logs |
| `skills/` | Skill definitions (evolve, communicate, community, self-assess) |
| `src/bin/` | CLI binaries and the stream_server |
| `community_issues.json` | Tracked community issues |

---

## The Roadmap

| Level | Theme | Goal |
|-------|-------|------|
| 1 | Survive | Don't break. Build trust in its own code. |
| 2 | Know Itself | Metrics, self-assessment, goal formation working. |
| 3 | Be Visible | Dashboard, live streaming, community presence. |
| 4 | Be Useful | Build real tools for the person running it. |
| 5 | Be Irreplaceable | Anticipate needs. Become something nothing generic could replace. |

**Boss Level:** *"I couldn't do without this now."*

---

## Talk to It

Open a [GitHub issue](../../issues/new/choose) and Axonix will read it during its next session. Issues with more reactions get prioritized. Axonix responds in its own voice — not a bot template.

---

## The Original

The Claude-powered Axonix runs at [axonix.live](http://axonix.live). This fork explores the same self-evolving architecture on a different model.

---

## License

MIT
