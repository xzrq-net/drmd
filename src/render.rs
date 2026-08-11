use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};
use pulldown_cmark::{Options, Parser};
use std::fmt::Write;
use std::path::Path;

/// Characters that would change a link's meaning if left raw in an href
/// (query/fragment starts, quotes, percent itself). `/` stays literal so
/// multi-segment paths pass through.
const HREF_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'\'')
    .add(b'<')
    .add(b'>')
    .add(b'#')
    .add(b'?')
    .add(b'%')
    .add(b'`')
    .add(b'&')
    .add(b'\\');

pub fn encode_href(path: &str) -> String {
    utf8_percent_encode(path, HREF_SET).to_string()
}

pub fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

pub fn markdown_body(source: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_FOOTNOTES);
    let parser = Parser::new_ext(source, options);
    let mut html = String::with_capacity(source.len() * 2);
    pulldown_cmark::html::push_html(&mut html, parser);
    html
}

/// Wrap rendered content in the page chrome: stylesheet, live-reload script.
pub fn page(title: &str, body: &str) -> String {
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{title}</title>
<link rel="stylesheet" href="/__assets/style.css">
</head>
<body>
<main>
{body}
</main>
<script src="/__assets/reload.js"></script>
</body>
</html>
"#,
        title = escape_html(title),
    )
}

pub struct ListingEntry {
    pub name: String,
    pub is_dir: bool,
}

/// Directory listing page body: subdirectories and markdown files.
pub fn listing_body(rel: &Path, entries: &[ListingEntry]) -> String {
    let mut body = String::new();
    let heading = if rel.as_os_str().is_empty() {
        "/".to_string()
    } else {
        format!("/{}/", rel.display())
    };
    write!(body, "<h1>{}</h1>\n<ul>\n", escape_html(&heading)).unwrap();
    if !rel.as_os_str().is_empty() {
        body.push_str("<li><a href=\"..\">..</a></li>\n");
    }
    for entry in entries {
        let name = escape_html(&entry.name);
        // "./" keeps names like "javascript:x" from parsing as a scheme.
        let href = encode_href(&entry.name);
        if entry.is_dir {
            writeln!(body, "<li><a href=\"./{href}/\">{name}/</a></li>").unwrap();
        } else {
            writeln!(body, "<li><a href=\"./{href}\">{name}</a></li>").unwrap();
        }
    }
    body.push_str("</ul>\n");
    body
}
