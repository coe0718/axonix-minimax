# Evolve Proposals

Operator should apply these proposals manually to scripts/evolve.sh.

---

## Proposal 1

**Problem:** METRICS.md only has 1 session recorded despite record_metrics being called in evolve.sh line 204. The binary likely fails because `cargo run --bin record_metrics` is run from the outer evolve.sh context after the container session ends, meaning it runs against the pre-session state.

**Proposed change to scripts/evolve.sh, around line 204:**

Move the `record_metrics` call to run AFTER all commits are done, and ensure it uses the committed state:

```bash
# ── Step 4b: Record session metrics ──
echo "→ Recording metrics..."
# record_metrics must run after all commits so --from-sha reflects actual diff
cargo run --bin record_metrics --quiet -- --day "$DAY" --date "$DATE" ${SESSION_START_SHA:+--from-sha "$SESSION_START_SHA"} 2>/dev/null \
    && echo "  Metrics recorded." \
    || echo "  Metrics recording failed (non-fatal)."
git add METRICS.md
if ! git diff --cached --quiet; then
    git commit -m "chore: Day $DAY S$SESSION metrics"
fi
```

The key insight: `SESSION_START_SHA` is set before the session runs (line 82), but if the session reverts source changes (line 190), `git diff` from that SHA will show nothing. The fix should either:
1. Always commit before running record_metrics (already done at lines 193-200), or
2. Always diff from SESSION_START_SHA (even if it's 0 or empty when the session fails)
3. Check that METRICS.md actually gets the new row by verifying row count increases

If the issue is that `cargo run --bin record_metrics` fails because of the container environment (no cargo, wrong working directory, etc.), wrap it in a check:
```bash
if command -v cargo &>/dev/null; then
    cargo run --bin record_metrics ...
fi
```
