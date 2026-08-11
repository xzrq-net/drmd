# Log

## 2026-08-10 — initial implementation (t7afxv)

Built the whole tool in one pass: `drmd <file-or-dir>` serves rendered
markdown on 127.0.0.1, ephemeral port by default, best-effort browser open.

Decisions:

- pulldown-cmark for parsing (tables, strikethrough, task lists, footnotes);
  axum + SSE (`/__reload`) + notify for live reload; sessionStorage keyed by
  pathname for scroll restore.
- Two styles as static CSS served from `/__assets/`: `classy` (serif body,
  dark-mode aware, styled tables) and `barebones` (browser defaults plus
  table borders). CLI `--style` sets the default; `?style=` overrides per
  request; a floating toolbar link toggles.
- Single-file mode serves the file at `/` and its parent dir as raw files so
  relative images work, but refuses directory listings there.

Codex (ception) adversarial review found one real killer: notify Access
events fire on every page render, so each browser reload triggered the next —
infinite reload loop. Fixed by filtering `EventKind::Access`. Also fixed from
review: hrefs/redirects now percent-encoded (`./`-prefixed links so
`javascript:foo.md` can't parse as a scheme), redirects are 307 and preserve
`?style=`, watch() failure downgraded to a warning, `.md`/`.markdown`
case-insensitive, explicit file target renders regardless of extension.

A second codex pass on the fix delta caught `&` and `\` missing from the
href encode set (entity mangling, WHATWG backslash-as-slash) — added. It also
noted that a mid-failure recursive watch keeps its earlier registrations, so
the failure warning now says coverage may be partial rather than disabled.

Accepted as designed for a localhost personal-preview threat model: raw HTML
passthrough (inline HTML in your own markdown is a feature, so `<script>` in
a viewed file executes), symlinks followed out of the root, hidden/non-md
files fetchable by direct URL, brief SSE window where a save during page load
is missed.

## 2026-08-10 — style machinery removed (teepma)

User looked at both grades and kept barebones. Deleted classy.css, the
`--style` flag, `?style=` override, and the toolbar toggle; barebones.css
renamed to style.css and hardcoded. `page()` and the handlers lost their
style parameter.
