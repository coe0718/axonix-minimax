## Proposal 1

**Problem**: METRICS.md always shows 0 files changed and 0 lines added/removed, making session metrics useless.

**Root cause**: In `evolve.sh`, the sequence is:
1. Agent makes changes
2. `git add -A && git commit` — all code changes committed
3. `cargo run --bin record_metrics -- --from-sha SESSION_START_SHA` — diffs against pre-session SHA

But step 2 already committed everything, so step 3 finds nothing to diff. METRICS.md shows zeros for every session.

**Proposed fix**: In `evolve.sh`, move the `record_metrics` call to BEFORE `git add -A`. This way it can diff the actual uncommitted changes.

Change this section:
```bash
# Commit any remaining uncommitted changes (journal, roadmap, day counter, etc.)
git add -A
if ! git diff --cached --quiet; then
    git commit -m "chore: Day $DAY Session $SESSION wrap-up"
    echo "  Committed session wrap-up."
else
    echo "  No uncommitted changes remaining."
fi

# ── Step 4b: Record session metrics ──
echo "→ Recording metrics..."
cargo run --bin record_metrics --quiet -- --day "$DAY" --date "$DATE" ${SESSION_START_SHA:+--from-sha "$SESSION_START_SHA"} 2>/dev/null \
    && echo "  Metrics recorded." \
    || echo "  Metrics recording failed (non-fatal)."
git add METRICS.md
if ! git diff --cached --quiet; then
    git commit -m "chore: Day $DAY S$SESSION metrics"
fi
```

To this:
```bash
# ── Step 4b: Record session metrics ──
# Do this BEFORE git add -A so we can diff actual uncommitted changes
echo "→ Recording metrics..."
cargo run --bin record_metrics --quiet -- --day "$DAY" --date "$DATE" ${SESSION_START_SHA:+--from-sha "$SESSION_START_SHA"} 2>/dev/null \
    && echo "  Metrics recorded." \
    || echo "  Metrics recording failed (non-fatal)."
git add METRICS.md
if ! git diff --cached --quiet; then
    git commit -m "chore: Day $DAY S$SESSION metrics"
fi

# ── Commit remaining uncommitted changes (journal, roadmap, day counter, etc.) ──
git add -A
if ! git diff --cached --quiet; then
    git commit -m "chore: Day $DAY Session $SESSION wrap-up"
    echo "  Committed session wrap-up."
else
    echo "  No uncommitted changes remaining."
fi
```

This reordering ensures record_metrics sees the actual diff before everything is committed.
