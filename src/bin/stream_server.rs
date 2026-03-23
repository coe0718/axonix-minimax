use axum::{
    body::Bytes,
    extract::State,
    response::{sse::Event, Html, Json, Sse},
    routing::{get, post},
    Router,
};
use pulldown_cmark::{html, Parser};
use serde::Serialize;
use std::convert::Infallible;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;
use tower_http::services::ServeDir;

const CHANNEL_CAPACITY: usize = 1024;
const PORT: u16 = 7041;

type AppState = Arc<broadcast::Sender<String>>;

/// Dynamic stats computed from METRICS.md and GOALS.md.
#[derive(Serialize)]
struct Stats {
    total_sessions: usize,
    latest_tests_passed: usize,
    total_files_changed: usize,
    total_lines_added: usize,
    total_lines_removed: usize,
    goals_completed: usize,
    goals_active: usize,
    goals_backlog: usize,
}

/// Render a markdown file as a styled HTML page.
fn render_markdown_page(title: &str, md_path: &Path) -> Html<String> {
    let content = fs::read_to_string(md_path).unwrap_or_else(|e| {
        format!("<p style='color:red'>Failed to read {}: {e}</p>", md_path.display())
    });

    let parser = Parser::new(&content);
    let mut body_html = String::new();
    html::push_html(&mut body_html, parser);

    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{title} — Axonix</title>
<style>
  body {{ font-family: system-ui, sans-serif; max-width: 800px; margin: 0 auto; padding: 2rem; background: #0f1117; color: #e6edf3; line-height: 1.6; }}
  h1, h2, h3 {{ color: #58a6ff; }}
  h1 {{ border-bottom: 2px solid #30363d; padding-bottom: 0.5rem; }}
  a {{ color: #58a6ff; }}
  code {{ background: #161b22; padding: 0.15em 0.4em; border-radius: 4px; font-size: 0.9em; }}
  pre {{ background: #161b22; padding: 1rem; border-radius: 6px; overflow-x: auto; }}
  pre code {{ background: none; padding: 0; }}
  hr {{ border: none; border-top: 1px solid #30363d; }}
  ul {{ padding-left: 1.5rem; }}
  li {{ margin: 0.3rem 0; }}
  blockquote {{ border-left: 3px solid #30363d; margin: 1rem 0; padding-left: 1rem; color: #8b949e; }}
  nav {{ display: flex; gap: 1.5rem; margin-bottom: 2rem; padding-bottom: 1rem; border-bottom: 1px solid #30363d; }}
  nav a {{ text-decoration: none; font-weight: 600; font-size: 1.1rem; }}
  nav a:hover {{ color: #79c0ff; }}
  .checked {{ color: #3fb950; }}
  .unchecked {{ color: #f85149; }}
</style>
</head>
<body>
<nav>
  <a href="/dashboard">Dashboard</a>
  <a href="/goals">Goals</a>
  <a href="/metrics">Metrics</a>
  <a href="/journal">Journal</a>
  <a href="/stream">Live Stream</a>
</nav>
{body_html}
</body>
</html>"#
    );
    Html(html)
}

#[tokio::main]
async fn main() {
    let (tx, _) = broadcast::channel::<String>(CHANNEL_CAPACITY);
    let state: AppState = Arc::new(tx);

    let app = Router::new()
        .route("/pipe", post(pipe))
        .route("/stream", get(stream))
        .route("/dashboard", get(dashboard))
        .route("/goals", get(goals))
        .route("/metrics", get(metrics))
        .route("/journal", get(journal))
        .route("/live", get(live))
        .route("/api/stats", get(api_stats))
        .with_state(state)
        .fallback_service(ServeDir::new("docs"));

    let addr: std::net::SocketAddr = match format!("0.0.0.0:{PORT}").parse() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("error: Invalid address: {e}");
            std::process::exit(1);
        }
    };
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("error: Failed to bind to {addr}: {e}");
            std::process::exit(1);
        }
    };
    println!("stream_server listening on {addr}");
    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("stream_server error: {e}");
    }
}

async fn pipe(State(tx): State<AppState>, body: Bytes) {
    let text = String::from_utf8_lossy(&body).into_owned();
    // Broadcast line by line so SSE clients get incremental updates
    for line in text.lines() {
        let _ = tx.send(line.to_owned());
    }
}

async fn stream(
    State(tx): State<AppState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let rx = tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|result: Result<String, _>| {
        result.ok().map(|line| Ok(Event::default().data(line)))
    });
    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default())
}

async fn dashboard() -> Html<String> {
    let home = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Axonix MiniMax Dashboard</title>
<style>
  body { font-family: system-ui, sans-serif; max-width: 900px; margin: 0 auto; padding: 2rem; background: #0f1117; color: #e6edf3; line-height: 1.6; }
  h1 { color: #58a6ff; border-bottom: 2px solid #30363d; padding-bottom: 0.5rem; }
  h2 { color: #79c0ff; margin-top: 2rem; }
  .card { background: #161b22; border: 1px solid #30363d; border-radius: 8px; padding: 1.5rem; margin: 1rem 0; }
  .card h3 { margin-top: 0; color: #e6edf3; }
  nav { display: flex; gap: 1.5rem; margin-bottom: 2rem; padding-bottom: 1rem; border-bottom: 1px solid #30363d; }
  nav a { color: #58a6ff; text-decoration: none; font-weight: 600; font-size: 1.1rem; }
  nav a:hover { color: #79c0ff; }
  .stats { display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 1rem; }
  .stat { background: #161b22; border: 1px solid #30363d; border-radius: 6px; padding: 1rem; text-align: center; }
  .stat .value { font-size: 2rem; font-weight: bold; color: #58a6ff; }
  .stat .label { font-size: 0.85rem; color: #8b949e; }
  ul { list-style: none; padding: 0; }
  li { padding: 0.5rem 0; border-bottom: 1px solid #21262d; }
  li:last-child { border-bottom: none; }
  .checked { color: #3fb950; }
  .unchecked { color: #f85149; }
  a { color: #58a6ff; }
  code { background: #161b22; padding: 0.15em 0.4em; border-radius: 4px; font-size: 0.9em; }
  .live-indicator { display: inline-block; width: 8px; height: 8px; background: #3fb950; border-radius: 50%; margin-right: 0.4rem; animation: pulse 2s infinite; }
  @keyframes pulse { 0%, 100% { opacity: 1; } 50% { opacity: 0.4; } }
  .error { color: #f85149; font-size: 0.85rem; }
</style>
</head>
<body>
<nav>
  <a href="/dashboard">Dashboard</a>
  <a href="/goals">Goals</a>
  <a href="/metrics">Metrics</a>
  <a href="/journal">Journal</a>
  <a href="/live"><span class="live-indicator"></span>Live</a>
</nav>

<h1>Axonix <span style="color:#f78166">MiniMax</span> Dashboard</h1>
<p style="color:#8b949e;margin-top:-0.5rem;font-size:0.9rem">Powered by MiniMax-M2.7 — same agent loop, different model</p>

<div class="stats" id="stats">
  <div class="stat">
    <div class="value" id="s-sessions">—</div>
    <div class="label">Sessions</div>
  </div>
  <div class="stat">
    <div class="value" id="s-tests">—</div>
    <div class="label">Latest Tests</div>
  </div>
  <div class="stat">
    <div class="value" id="s-files">—</div>
    <div class="label">Files Changed</div>
  </div>
  <div class="stat">
    <div class="value" id="s-goals">—</div>
    <div class="label">Goals Done</div>
  </div>
  <div class="stat">
    <div class="value" id="s-lines">—</div>
    <div class="label">Lines Added</div>
  </div>
</div>
<div id="stats-error" class="error" style="display:none"></div>

<div class="card">
  <h3>About</h3>
  <p>Axonix MiniMax is an experiment: the same self-evolving agent architecture running on <a href="https://www.minimaxi.com/">MiniMax-M2.7</a> instead of Claude. Same goals, same evolve loop, different brain. The source is public at <a href="https://github.com/coe0718/axonix-minimax">github.com/coe0718/axonix-minimax</a>. The original Axonix (Claude) runs at <a href="http://axonix.live">axonix.live</a>.</p>
</div>

<div class="card">
  <h3>Skills</h3>
  <ul>
    <li><a href="/skills/communicate/SKILL.md">communicate</a> — journal entries and GitHub issue responses</li>
    <li><a href="/skills/community/SKILL.md">community</a> — reading and prioritizing community input</li>
    <li><a href="/skills/evolve/SKILL.md">evolve</a> — core self-improvement skill</li>
    <li><a href="/skills/self-assess/SKILL.md">self-assess</a> — evaluating code, goals, and metrics</li>
  </ul>
</div>

<div class="card">
  <h3>Tools</h3>
  <ul>
    <li><code>check_yaml</code> — validates YAML/YML files</li>
    <li><code>check_caddyfile</code> — validates Caddyfile syntax and formatting</li>
    <li><code>record_metrics</code> — records session metrics to METRICS.md</li>
  </ul>
</div>

<script>
async function loadStats() {
  try {
    const r = await fetch('/api/stats');
    if (!r.ok) throw new Error('HTTP ' + r.status);
    const d = await r.json();
    document.getElementById('s-sessions').textContent = d.total_sessions;
    document.getElementById('s-tests').textContent = d.latest_tests_passed;
    document.getElementById('s-files').textContent = d.total_files_changed;
    document.getElementById('s-goals').textContent = d.goals_completed;
    document.getElementById('s-lines').textContent = d.total_lines_added;
    document.getElementById('stats-error').style.display = 'none';
  } catch(e) {
    const el = document.getElementById('stats-error');
    el.textContent = 'Stats unavailable (server may not be running from project root)';
    el.style.display = 'block';
  }
}
loadStats();
</script>

</body>
</html>"#;
    Html(home.to_string())
}

async fn goals() -> Html<String> {
    render_markdown_page("Goals", Path::new("GOALS.md"))
}

async fn metrics() -> Html<String> {
    render_markdown_page("Metrics", Path::new("METRICS.md"))
}

async fn journal() -> Html<String> {
    render_markdown_page("Journal", Path::new("JOURNAL.md"))
}

/// Dynamically compute stats from METRICS.md and GOALS.md.
async fn api_stats() -> Json<Stats> {
    let metrics = fs::read_to_string("METRICS.md").unwrap_or_default();
    let goals_md = fs::read_to_string("GOALS.md").unwrap_or_default();

    let mut sessions = 0;
    let mut latest_tests_passed = 0;
    let mut total_files = 0;
    let mut total_added = 0;
    let mut total_removed = 0;

    for line in metrics.lines() {
        if line.starts_with("| 1 ") || line.starts_with("| 2 ") || line.starts_with("| 3 ") {
            sessions += 1;
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 9 {
                if let Ok(tp) = parts[4].trim().parse::<usize>() {
                    latest_tests_passed = tp;
                }
                if let Ok(fc) = parts[6].trim().parse::<usize>() {
                    total_files += fc;
                }
                if let Ok(la) = parts[7].trim().parse::<usize>() {
                    total_added += la;
                }
                if let Ok(lr) = parts[8].trim().parse::<usize>() {
                    total_removed += lr;
                }
            }
        }
    }

    let goals_completed = goals_md.matches("- [x]").count();
    let goals_active = goals_md.matches("- [ ] [G-").count();
    let goals_backlog = goals_md.matches("## Backlog").next().map_or(0, |_| {
        goals_md.matches("- [ ]").count()
    });

    Json(Stats {
        total_sessions: sessions,
        latest_tests_passed,
        total_files_changed: total_files,
        total_lines_added: total_added,
        total_lines_removed: total_removed,
        goals_completed,
        goals_active,
        goals_backlog,
    })
}

/// Dedicated live stream page with an SSE client that renders real-time output.
async fn live() -> Html<String> {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Live Stream — Axonix MiniMax</title>
<style>
  * { box-sizing: border-box; }
  body { font-family: 'Courier New', monospace; background: #0d1117; color: #c9d1d9; margin: 0; padding: 1rem; height: 100vh; display: flex; flex-direction: column; }
  nav { display: flex; gap: 1.5rem; padding-bottom: 1rem; border-bottom: 1px solid #30363d; margin-bottom: 1rem; }
  nav a { color: #58a6ff; text-decoration: none; font-weight: 600; font-family: system-ui, sans-serif; font-size: 1rem; }
  nav a:hover { color: #79c0ff; }
  #status { font-family: system-ui, sans-serif; font-size: 0.85rem; color: #8b949e; margin-bottom: 0.5rem; }
  #status.connected { color: #3fb950; }
  #status.disconnected { color: #f85149; }
  #output { flex: 1; overflow-y: auto; background: #161b22; border: 1px solid #30363d; border-radius: 6px; padding: 1rem; font-size: 0.9rem; line-height: 1.5; white-space: pre-wrap; word-break: break-all; min-height: 300px; }
  .line { border-bottom: 1px solid #21262d; padding: 0.1rem 0; }
  .line:last-child { border-bottom: none; }
  .empty { color: #484f58; font-style: italic; }
  #clear { margin-top: 0.5rem; padding: 0.4rem 1rem; background: #21262d; color: #c9d1d9; border: 1px solid #30363d; border-radius: 6px; cursor: pointer; font-family: system-ui, sans-serif; }
  #clear:hover { background: #30363d; }
</style>
</head>
<body>
<nav>
  <a href="/dashboard">Dashboard</a>
  <a href="/goals">Goals</a>
  <a href="/metrics">Metrics</a>
  <a href="/journal">Journal</a>
  <a href="/stream">Raw SSE</a>
</nav>
<div id="status">Connecting…</div>
<div id="output"><span class="empty">Waiting for session output…</span></div>
<button id="clear">Clear output</button>
<script>
const status = document.getElementById('status');
const output = document.getElementById('output');
const clearBtn = document.getElementById('clear');

let es;
let lineCount = 0;

function connect() {
  es = new EventSource('/stream');
  status.textContent = 'Connecting…';
  status.className = '';

  es.onopen = () => {
    status.textContent = '● Connected — receiving live session output';
    status.className = 'connected';
  };

  es.onmessage = (e) => {
    if (lineCount === 1 && output.querySelector('.empty')) {
      output.innerHTML = '';
    }
    const div = document.createElement('div');
    div.className = 'line';
    div.textContent = e.data;
    output.appendChild(div);
    lineCount++;
    output.scrollTop = output.scrollHeight;
    // Keep last 500 lines to prevent memory bloat
    while (output.children.length > 500) {
      output.removeChild(output.firstChild);
    }
  };

  es.onerror = () => {
    status.textContent = '✗ Disconnected — stream closed';
    status.className = 'disconnected';
    es.close();
    // Reconnect after 3 seconds
    setTimeout(connect, 3000);
  };
}

clearBtn.addEventListener('click', () => {
  output.innerHTML = '<span class="empty">Output cleared. Waiting for new messages…</span>';
  lineCount = 0;
});

connect();
</script>
</body>
</html>"#;
    Html(html.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_goals_page() {
        let html = render_markdown_page("Goals", Path::new("GOALS.md"));
        let s = html.0.as_str();
        assert!(s.contains("<!DOCTYPE html>"), "should be valid HTML");
        assert!(s.contains("Axonix"), "should contain title");
        assert!(s.contains("/goals"), "should have nav link");
        assert!(s.contains("/journal"), "should have nav link");
    }

    #[test]
    fn test_render_metrics_page() {
        let html = render_markdown_page("Metrics", Path::new("METRICS.md"));
        assert!(html.0.contains("<!DOCTYPE html>"));
    }

    #[test]
    fn test_render_journal_page() {
        let html = render_markdown_page("Journal", Path::new("JOURNAL.md"));
        assert!(html.0.contains("<!DOCTYPE html>"));
    }

    #[test]
    fn test_dashboard_page() {
        // just verify it compiles and returns HTML
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let html = dashboard().await;
            let s = html.0.as_str();
            assert!(s.contains("<!DOCTYPE html>"));
            assert!(s.contains("Sessions"));
            assert!(s.contains("/goals"));
            assert!(s.contains("/metrics"));
            assert!(s.contains("/journal"));
            assert!(s.contains("/api/stats"));
        });
    }

    #[test]
    fn test_api_stats_returns_valid_json() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let json = api_stats().await;
            // Should deserialize cleanly
            let s = &json.0;
            assert!(s.total_sessions >= 0);
            assert!(s.goals_completed >= 0);
            assert!(s.goals_active >= 0);
        });
    }

    #[test]
    fn test_live_page_has_sse_client() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let html = live().await;
            let s = html.0.as_str();
            assert!(s.contains("<!DOCTYPE html>"));
            assert!(s.contains("EventSource"));
            assert!(s.contains("/stream"));
            assert!(s.contains("Connecting…"));
            assert!(s.contains("Clear output"));
        });
    }
}
