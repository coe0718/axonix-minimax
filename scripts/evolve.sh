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
ISSUES_REPO="${ISSUES_REPO:-coe0718/axonix-minimax}"
TIMEOUT="${TIMEOUT:-600}"
DATE=$(date +%Y-%m-%d)

# DAY_COUNT format: "N YYYY-MM-DD" — N is calendar days, date is last run date
COUNT_RAW=$(cat DAY_COUNT 2>/dev/null || echo "0 ")
STORED_DAY=$(echo "$COUNT_RAW" | awk '{print $1}')
STORED_DATE=$(echo "$COUNT_RAW" | awk '{print $2}')

if [ "$STORED_DATE" = "$DATE" ]; then
    DAY=$STORED_DAY
    SESSION=$(($(cat SESSION_COUNT 2>/dev/null || echo 0) + 1))
else
    DAY=$((STORED_DAY + 1))
    SESSION=1
    echo "$DAY $DATE" > DAY_COUNT
fi
echo "$SESSION" > SESSION_COUNT

echo "=== Day $DAY, Session $SESSION: $DATE ==="
echo "Provider: MiniMax (MiniMax-M2.7 via Anthropic-compat API)"
echo "Timeout: ${TIMEOUT}s"
echo ""

# Warn if a pending operator proposal exists
if [ -f EVOLVE_PROPOSED.md ]; then
    echo "  ⚠️  EVOLVE_PROPOSED.md exists — operator action required before this is applied."
fi

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
    gh issue list --repo "$ISSUES_REPO" \
        --state open \
        --limit 50 \
        --json number,title,body,labels,reactionGroups \
        > /tmp/issues_raw.json 2>/dev/null || true

    python3 scripts/format_issues.py /tmp/issues_raw.json > "$ISSUES_FILE" 2>/dev/null || echo "No issues found." > "$ISSUES_FILE"
    echo "  $(grep -c '^### Issue' "$ISSUES_FILE" 2>/dev/null || echo 0) issues loaded."
else
    echo "  gh CLI not available. Skipping issue fetch."
    echo "No issues available (gh CLI not installed)." > "$ISSUES_FILE"
fi
echo ""

# Snapshot HEAD before session for accurate diff stats in record_metrics
SESSION_START_SHA=$(git rev-parse HEAD 2>/dev/null || echo "")

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
Today is Day $DAY, Session $SESSION ($DATE). You are Axonix, a self-evolving coding agent.

IMPORTANT: You must use tools to do all work. Do not just think or plan — act.
Your FIRST action must be to call read_file with path="IDENTITY.md".

=== PHASE 1: Read context ===

Read these files in order (call the tool for each):
1. IDENTITY.md — who you are and your values
2. GOALS.md — active goals and backlog
3. JOURNAL.md — recent session history (last 3 entries are enough)
4. ISSUES_TODAY.md — community requests

Read src/main.rs and other src/ files only when directly relevant to your task.
Do NOT read all .rs files upfront.

=== PHASE 2: Self-Assessment ===

Run: cargo build && cargo test 2>&1 | grep -E "(^test result|FAILED|^error\[)"
Report the exact test count. Note any bugs or friction.

=== PHASE 3: Decide ===

Choose what to work on. Priority:
1. Crash or data loss bug you just discovered
2. Community issue from ISSUES_TODAY.md with most reactions
3. Active goal from GOALS.md
4. UX friction or missing error handling
5. Whatever makes you most useful to the person running you

=== PHASE 4: Journal + Goals (DO THIS BEFORE CODING) ===

Step 4a — Prepend an entry at the top of JOURNAL.md:
## Day $DAY, Session $SESSION — [title]
[2-4 sentences: what you plan to do and why]

Step 4b — Update GOALS.md:
- For any goal you verified is already done in code: mark [x] now
- If Active section is empty, promote one item from Backlog
- Commit: git add JOURNAL.md GOALS.md && git commit -m "docs: Day $DAY S$SESSION plan"

=== PHASE 5: Implement ===

- Use edit_file for surgical changes
- Run: cargo build && cargo test after each change
- If build fails, revert: git checkout -- src/
- After each successful change: git add -A && git commit -m "type(scope): description"
- Keep improving until you run out of good ideas

=== PHASE 6: Issue response (if you addressed a community issue) ===

Create ISSUE_RESPONSE.md:
issue_number: [N]
status: fixed|partial|wontfix
comment: [2-3 sentence response]

=== PHASE 7: Wrap up ===

Update JOURNAL.md entry with what actually happened (replace plan with results).
Update GOALS.md: mark completed goals [x], update progress notes.
Run: cargo build && cargo test — must pass before you finish.

== EVOLVE_PROPOSED.md RULES ==

scripts/evolve.sh is READ-ONLY inside the container. You cannot modify it directly.
If you need to propose a change to evolve.sh:
- If EVOLVE_PROPOSED.md does NOT exist: create it with your proposal as "## Proposal 1"
- If EVOLVE_PROPOSED.md ALREADY EXISTS: append your proposal as the next numbered section
- NEVER overwrite or delete existing proposals — the operator may not have applied them yet
- Describe the change clearly enough that the operator can apply it manually

Begin now. Call read_file with path="IDENTITY.md" immediately.
PROMPT

export API_KEY="${MINIMAX_API_KEY}"

${TIMEOUT_CMD:+$TIMEOUT_CMD "$TIMEOUT"} \
    cargo run --bin axonix -- --model MiniMax-M2.7 --skills ./skills \
    < "$PROMPT_FILE" 2>&1 \
    | tee /tmp/session.log

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

# ── Step 5: Handle issue responses ──
for RESPONSE_FILE in ISSUE_RESPONSE*.md; do
    [ -f "$RESPONSE_FILE" ] || continue
    echo ""
    echo "→ Posting issue response from $RESPONSE_FILE..."

    ISSUE_NUM=$(grep "^issue_number:" "$RESPONSE_FILE" | awk '{print $2}' || true)
    STATUS=$(grep "^status:" "$RESPONSE_FILE" | awk '{print $2}' || true)
    COMMENT=$(sed -n '/^comment:/,$ p' "$RESPONSE_FILE" | sed '1s/^comment: //' || true)

    if [ -n "$ISSUE_NUM" ] && command -v gh &>/dev/null; then
        gh issue comment "$ISSUE_NUM" \
            --repo "$ISSUES_REPO" \
            --body "🤖 **Day $DAY, Session $SESSION** (axonix-minimax experiment)

$COMMENT

Commit: $(git rev-parse --short HEAD)" || true

        if [ "$STATUS" = "fixed" ]; then
            gh issue close "$ISSUE_NUM" --repo "$ISSUES_REPO" || true
            echo "  Closed issue #$ISSUE_NUM"
        else
            echo "  Commented on issue #$ISSUE_NUM (status: $STATUS)"
        fi
    fi

    rm -f "$RESPONSE_FILE"
done

# ── Step 6: Enforce non-empty commit body before push ──
LAST_COMMIT_BODY=$(git log -1 --format="%b" | tr -d '[:space:]')
if [ -z "$LAST_COMMIT_BODY" ]; then
    echo "  WARNING: last commit has no body — amending before push"
    LAST_SUBJECT=$(git log -1 --format="%s")
    COMMIT_DATE=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    git commit --amend --no-edit -m "${LAST_SUBJECT}

Auto-generated body: session wrap-up at ${COMMIT_DATE}."
fi

# ── Step 7: Push ──
echo ""
echo "→ Pushing..."
git push || echo "  Push failed (maybe no remote or auth issue)"

echo ""
echo "=== Day $DAY, Session $SESSION complete ==="
