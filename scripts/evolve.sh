#!/bin/bash
# scripts/evolve.sh — One evolution cycle powered by GitHub Copilot Codex.
#
# Usage:
#   ./scripts/evolve.sh
#
# Prerequisites:
#   npm install -g @openai/codex
#   codex --provider github-copilot  (first run — completes device login flow)
#
# Environment:
#   MINIMAX_API_KEY — required
#   REPO            — GitHub repo (default: coe0718/axonix-minimax)
#   TIMEOUT         — Max session time in seconds (default: 600)

set -euo pipefail

# Ensure git trusts the workspace (needed when running as non-root with volume mount)
git config --global --add safe.directory /workspace 2>/dev/null || true
git config --global user.email "axonix@axonix.live" 2>/dev/null || true
git config --global user.name "Axonix" 2>/dev/null || true
if [ -n "${GIT_REMOTE_URL:-}" ]; then
    git remote set-url origin "$GIT_REMOTE_URL" 2>/dev/null || true
fi

REPO="${REPO:-coe0718/axonix-minimax}"
TIMEOUT="${TIMEOUT:-600}"
DAY=$(cat DAY_COUNT 2>/dev/null || echo 1)
DATE=$(date +%Y-%m-%d)

echo "=== Day $DAY: $DATE ==="
echo "Provider: MiniMax (MiniMax-M2.7 via Anthropic-compat API)"
echo "Timeout: ${TIMEOUT}s"
echo ""

# ── Step 1: Verify starting state ──
echo "→ Checking build..."
cargo build --quiet
cargo test --quiet
echo "  Build OK."
echo ""

# ── Step 2: Fetch GitHub issues ──
ISSUES_FILE="ISSUES_TODAY.md"
echo "→ Fetching community issues..."
if command -v gh &>/dev/null; then
    gh issue list --repo "$REPO" \
        --state open \
        --label "agent-input" \
        --limit 10 \
        --json number,title,body,labels,reactionGroups \
        > /tmp/issues_raw.json 2>/dev/null || true

    python3 scripts/format_issues.py /tmp/issues_raw.json > "$ISSUES_FILE" 2>/dev/null || echo "No issues found." > "$ISSUES_FILE"
    echo "  $(grep -c '^### Issue' "$ISSUES_FILE" 2>/dev/null || echo 0) issues loaded."
else
    echo "  gh CLI not available. Skipping issue fetch."
    echo "No issues available (gh CLI not installed)." > "$ISSUES_FILE"
fi
echo ""

# ── Step 3: Run evolution session ──
echo "→ Starting evolution session (Codex)..."
echo ""

TIMEOUT_CMD="timeout"
if ! command -v timeout &>/dev/null; then
    if command -v gtimeout &>/dev/null; then
        TIMEOUT_CMD="gtimeout"
    else
        TIMEOUT_CMD=""
    fi
fi

PROMPT_FILE=$(mktemp)
cat > "$PROMPT_FILE" <<PROMPT
Today is Day $DAY ($DATE). You are Axonix, a self-evolving coding agent.

IMPORTANT: You must use tools to do all work. Do not just think or plan — act.
Your FIRST action must be to call read_file with path="IDENTITY.md".

Step 1 — Use read_file to read these files in order (call the tool for each):
1. IDENTITY.md
2. src/main.rs
3. JOURNAL.md
4. ISSUES_TODAY.md

Step 2 — Self-Assessment:
Use read_file or bash to explore the codebase. Run bash commands to test your
own functionality. Note bugs, friction, or missing capabilities.

Step 3 — Decide what to improve:
Priority order:
1. Crash or data loss bug you just discovered
2. Community issue from ISSUES_TODAY.md with most reactions
3. UX friction or missing error handling
4. Whatever makes you most useful to real developers

Step 4 — Implement:
- Use edit_file for surgical changes to existing files
- Use bash to run: cargo build && cargo test
- If build fails, revert with: bash "git checkout -- src/"
- After each successful change: bash "git add -A && git commit -m 'Day $DAY: description'"
- Keep making improvements until you run out of good ideas

Step 5 — Update JOURNAL.md:
Use write_file or edit_file to prepend an entry at the top:
## Day $DAY — [title]
[2-4 sentences: what you tried, what worked, what didn't, what's next]

Step 6 — Issue response (if you addressed a community issue):
Use write_file to create ISSUE_RESPONSE.md:
issue_number: [N]
status: fixed|partial|wontfix
comment: [2-3 sentence response]

Begin now. Call read_file with path="IDENTITY.md" immediately.
PROMPT

export API_KEY="${MINIMAX_API_KEY}"

${TIMEOUT_CMD:+$TIMEOUT_CMD "$TIMEOUT"} \
    cargo run --bin axonix -- --model MiniMax-M2.7 --skills ./skills \
    < "$PROMPT_FILE" 2>&1 \
    | tee /tmp/session.log
echo "--- session log tail ---"
tail -20 /tmp/session.log || true

rm -f "$PROMPT_FILE"

echo ""
echo "→ Session complete. Checking results..."

# ── Step 4: Verify build and handle leftovers ──
if cargo build --quiet 2>/dev/null && cargo test --quiet 2>/dev/null; then
    echo "  Build: PASS"
else
    echo "  Build: FAIL — reverting source changes"
    git checkout -- src/
fi

# Increment day counter
echo "$((DAY + 1))" > DAY_COUNT

# Commit any remaining uncommitted changes (journal, roadmap, day counter, etc.)
git add -A
if ! git diff --cached --quiet; then
    git commit -m "Day $DAY: session wrap-up"
    echo "  Committed session wrap-up."
else
    echo "  No uncommitted changes remaining."
fi

# ── Step 5: Handle issue response ──
if [ -f ISSUE_RESPONSE.md ]; then
    echo ""
    echo "→ Posting issue response..."

    ISSUE_NUM=$(grep "^issue_number:" ISSUE_RESPONSE.md | awk '{print $2}' || true)
    STATUS=$(grep "^status:" ISSUE_RESPONSE.md | awk '{print $2}' || true)
    COMMENT=$(sed -n '/^comment:/,$ p' ISSUE_RESPONSE.md | sed '1s/^comment: //' || true)

    if [ -n "$ISSUE_NUM" ] && command -v gh &>/dev/null; then
        gh issue comment "$ISSUE_NUM" \
            --repo "$REPO" \
            --body "🤖 **Day $DAY**

$COMMENT

Commit: $(git rev-parse --short HEAD)" || true

        if [ "$STATUS" = "fixed" ]; then
            gh issue close "$ISSUE_NUM" --repo "$REPO" || true
            echo "  Closed issue #$ISSUE_NUM"
        else
            echo "  Commented on issue #$ISSUE_NUM (status: $STATUS)"
        fi
    fi

    rm -f ISSUE_RESPONSE.md
fi

# ── Step 6: Push ──
echo ""
echo "→ Pushing..."
git push || echo "  Push failed (maybe no remote or auth issue)"

echo ""
echo "=== Day $DAY complete ==="
