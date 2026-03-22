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

REPO="${REPO:-coe0718/axonix-minimax}"
TIMEOUT="${TIMEOUT:-600}"
DAY=$(cat DAY_COUNT 2>/dev/null || echo 1)
DATE=$(date +%Y-%m-%d)

echo "=== Day $DAY: $DATE ==="
echo "Provider: MiniMax (minimax2.7 via Anthropic-compat API)"
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
Today is Day $DAY ($DATE).

Read these files in this order:
1. IDENTITY.md (who you are and your rules)
2. src/main.rs (your current source code — this is YOU)
3. JOURNAL.md (your recent history — last 10 entries)
4. ISSUES_TODAY.md (community requests)

=== PHASE 1: Self-Assessment ===

Read your own source code carefully. Then try a small task to test
yourself — for example, read a file, edit something, run a command.
Note any friction, bugs, crashes, or missing capabilities.

=== PHASE 2: Review Community Issues ===

Read ISSUES_TODAY.md. These are real people asking you to improve.
Issues with more 👍 reactions should be prioritized higher.

=== PHASE 3: Decide ===

Make as many improvements as you can this session. Prioritize:
1. Self-discovered crash or data loss bug
2. Community issue with most 👍 (if actionable today)
3. Self-discovered UX friction or missing error handling
4. Whatever you think will make you most useful to real developers

=== PHASE 4: Implement ===

For each improvement, follow the evolve skill rules:
- Write a test first if possible
- Use edit_file for surgical changes
- Run cargo build && cargo test after changes
- If build fails, try to fix it. If you can't, revert with: git checkout -- src/
- After each successful change, commit: git add -A && git commit -m "Day $DAY: <short description>"
- Then move on to the next improvement

=== PHASE 5: Journal ===

Write today's entry at the TOP of JOURNAL.md. Format:
## Day $DAY — [title]
[2-4 sentences: what you tried, what worked, what didn't, what's next]

=== PHASE 6: Issue Response ===

If you worked on a community GitHub issue, write to ISSUE_RESPONSE.md:
issue_number: [N]
status: fixed|partial|wontfix
comment: [your 2-3 sentence response to the person]

Now begin. Read IDENTITY.md first.
PROMPT

${TIMEOUT_CMD:+$TIMEOUT_CMD "$TIMEOUT"} \
    ANTHROPIC_BASE_URL=https://api.minimax.io/anthropic \
    ANTHROPIC_API_KEY="${MINIMAX_API_KEY}" \
    cargo run --quiet -- --model minimax2.7 --skills ./skills \
    < "$PROMPT_FILE" 2>&1 \
    | tee /tmp/session.log || true

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
