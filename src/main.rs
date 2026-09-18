mod render;

use anyhow::{Context, Result, bail};
use axum::Router;
use axum::extract::{Path as UrlPath, State};
use axum::http::{StatusCode, header};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::get;
use clap::Parser;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;

/// Serve a markdown file or directory as rendered HTML, live-reloading on
/// change.
#[derive(Parser)]
struct Cli {
    /// Markdown file or directory to serve
    path: PathBuf,
    /// Port to listen on (0 picks a free one)
    #[arg(short, long, default_value_t = 0)]
    port: u16,
    /// Don't open a browser
    #[arg(long)]
    no_open: bool,
}

struct App {
    /// Directory files are served from.
    root: PathBuf,
    /// In single-file mode, the file (inside `root`) served at `/`.
    single_file: Option<PathBuf>,
    reload_tx: broadcast::Sender<()>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let target = cli
        .path
        .canonicalize()
        .with_context(|| format!("{}", cli.path.display()))?;
    let (root, single_file) = if target.is_dir() {
        (target.clone(), None)
    } else {
        let parent = target.parent().context("file has no parent dir")?;
        (parent.to_path_buf(), Some(target.clone()))
    };

    let (reload_tx, _) = broadcast::channel(16);
    let _watcher = start_watcher(&target, single_file.is_none(), reload_tx.clone())?;

    let app = Arc::new(App {
        root,
        single_file,
        reload_tx,
    });
    let router = Router::new()
        .route("/", get(serve_path))
        .route("/{*path}", get(serve_path))
        .route("/__reload", get(sse_reload))
        .route("/__assets/{file}", get(serve_asset))
        .with_state(app);

    let listener = tokio::net::TcpListener::bind(("127.0.0.1", cli.port)).await?;
    let url = format!("http://{}/", listener.local_addr()?);
    println!("serving {} at {url}", target.display());
    if !cli.no_open {
        let open_url = url.clone();
        std::thread::spawn(move || {
            if let Err(err) = open::that(&open_url) {
                eprintln!("browser open failed: {err}");
            }
        });
    }
    axum::serve(listener, router).await?;
    Ok(())
}

/// Watch for changes and forward debounced reload signals. In single-file
/// mode only events touching the file count (editors churn out tempfile
/// events); in directory mode hidden paths (.git etc.) are ignored.
fn start_watcher(
    target: &Path,
    recursive: bool,
    reload_tx: broadcast::Sender<()>,
) -> Result<notify::RecommendedWatcher> {
    use notify::{RecursiveMode, Watcher};

    let single_file = (!recursive).then(|| target.to_path_buf());
    let watch_dir = match &single_file {
        Some(file) => file.parent().unwrap().to_path_buf(),
        None => target.to_path_buf(),
    };
    let root = watch_dir.clone();
    let relevant = move |event: &notify::Event| {
        event.paths.iter().any(|p| match &single_file {
            Some(file) => p == file,
            None => match p.strip_prefix(&root) {
                Ok(rel) => !rel
                    .components()
                    .any(|c| c.as_os_str().to_string_lossy().starts_with('.')),
                Err(_) => true,
            },
        })
    };

    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        // Access events fire on every page render; reacting to them would
        // make each reload trigger the next.
        if let Ok(event) = res
            && !matches!(event.kind, notify::EventKind::Access(_))
            && relevant(&event)
        {
            let _ = tx.send(());
        }
    })?;
    let mode = if recursive {
        RecursiveMode::Recursive
    } else {
        RecursiveMode::NonRecursive
    };
    if let Err(err) = watcher.watch(&watch_dir, mode) {
        eprintln!("watch failed, live reload may not cover everything: {err}");
    }

    std::thread::spawn(move || {
        while rx.recv().is_ok() {
            // Let the burst of events from a single save settle.
            while rx.recv_timeout(Duration::from_millis(100)).is_ok() {}
            let _ = reload_tx.send(());
        }
    });
    Ok(watcher)
}

async fn sse_reload(State(app): State<Arc<App>>) -> impl IntoResponse {
    let stream = BroadcastStream::new(app.reload_tx.subscribe())
        .map(|_| Ok::<_, std::convert::Infallible>(Event::default().data("reload")));
    Sse::new(stream).keep_alive(KeepAlive::default())
}

async fn serve_asset(UrlPath(file): UrlPath<String>) -> Response {
    let (body, mime) = match file.as_str() {
        "style.css" => (include_str!("../assets/style.css"), "text/css"),
        "reload.js" => (include_str!("../assets/reload.js"), "text/javascript"),
        "diagrams.js" => (include_str!("../assets/diagrams.js"), "text/javascript"),
        "mermaid.min.js" => (
            include_str!("../assets/vendor/mermaid.min.js"),
            "text/javascript",
        ),
        _ => return StatusCode::NOT_FOUND.into_response(),
    };
    ([(header::CONTENT_TYPE, mime)], body).into_response()
}

async fn serve_path(State(app): State<Arc<App>>, path: Option<UrlPath<String>>) -> Response {
    let rel = path.map(|UrlPath(p)| p).unwrap_or_default();
    match resolve(&app, &rel) {
        Ok(full) => serve_resolved(&app, &rel, &full).await,
        Err(err) => (StatusCode::NOT_FOUND, format!("{err}")).into_response(),
    }
}

/// Map a request path to a filesystem path under the root, rejecting
/// traversal.
fn resolve(app: &App, rel: &str) -> Result<PathBuf> {
    if rel.is_empty() {
        return Ok(match &app.single_file {
            Some(file) => file.clone(),
            None => app.root.clone(),
        });
    }
    let rel_path = Path::new(rel.trim_end_matches('/'));
    if !rel_path
        .components()
        .all(|c| matches!(c, Component::Normal(_)))
    {
        bail!("bad path");
    }
    Ok(app.root.join(rel_path))
}

async fn serve_resolved(app: &App, rel: &str, full: &Path) -> Response {
    if full.is_dir() {
        // Single-file mode is about one document; don't grow into a browser
        // for the parent tree.
        if app.single_file.is_some() {
            return (StatusCode::NOT_FOUND, "not found").into_response();
        }
        if !rel.is_empty() && !rel.ends_with('/') {
            let target = format!("/{}/", render::encode_href(rel));
            return Redirect::temporary(&target).into_response();
        }
        return match listing(rel, full) {
            Ok(page) => Html(page).into_response(),
            Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{err}")).into_response(),
        };
    }
    if !full.is_file() {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    }
    if is_markdown(full) || app.single_file.as_deref() == Some(full) {
        return match tokio::fs::read_to_string(full).await {
            Ok(source) => {
                let title = full.file_name().unwrap_or_default().to_string_lossy();
                let body = render::markdown_body(&source);
                Html(render::page(&title, &body)).into_response()
            }
            Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{err}")).into_response(),
        };
    }
    // Non-markdown files (images etc. referenced from pages) served raw.
    match tokio::fs::read(full).await {
        Ok(bytes) => ([(header::CONTENT_TYPE, guess_mime(full))], bytes).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{err}")).into_response(),
    }
}

fn listing(rel: &str, dir: &Path) -> Result<String> {
    let mut entries = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') {
            continue;
        }
        let is_dir = entry.file_type()?.is_dir();
        if is_dir || is_markdown(Path::new(&name)) {
            entries.push(render::ListingEntry { name, is_dir });
        }
    }
    entries.sort_by_key(|e| (!e.is_dir, e.name.to_lowercase()));
    let rel_path = Path::new(rel.trim_end_matches('/'));
    let body = render::listing_body(rel_path, &entries);
    let title = format!("/{rel}");
    Ok(render::page(&title, &body))
}

fn is_markdown(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| e.eq_ignore_ascii_case("md") || e.eq_ignore_ascii_case("markdown"))
}

fn guess_mime(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        "css" => "text/css",
        "js" => "text/javascript",
        "html" | "htm" => "text/html",
        "txt" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}
