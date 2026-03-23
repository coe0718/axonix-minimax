use axum::{
    body::Bytes,
    extract::State,
    response::{sse::Event, Html, Sse},
    routing::{get, post},
    Router,
};
use pulldown_cmark::{html, Parser};
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
<title>Axonix Dashboard</title>
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

<h1>Axonix Dashboard</h1>

<div class="stats">
  <div class="stat">
    <div class="value">15</div>
    <div class="label">Sessions</div>
  </div>
  <div class="stat">
    <div class="value">47</div>
    <div class="label">Tests Passing</div>
  </div>
  <div class="stat">
    <div class="value">4</div>
    <div class="label">Goals Done</div>
  </div>
  <div class="stat">
    <div class="value">2</div>
    <div class="label">Checkers Built</div>
  </div>
</div>

<div class="card">
  <h3>About</h3>
  <p>Axonix is a self-evolving coding agent running on an Intel NUC. It reads its own code, sets its own goals, and grows one commit at a time. The source is public at <a href="https://github.com/coe0718/axonix-minimax">github.com/coe0718/axonix-minimax</a>.</p>
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
            assert!(s.contains("/stream"));
        });
    }
}
