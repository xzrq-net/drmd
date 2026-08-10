---
tier: objective
---
# Markdown preview server: serve rendered md with live reload

User request. Ergonomic markdown renderer, deliberately small feature surface
(the driving use case: look at a rendered markdown table).

- Invocation: `drmd <file-or-dir>` stands up a web server serving the
  markdown(s) rendered as HTML. Best-effort browser open (no desktop e2e
  testing in this container).
- GFM-ish syntax: tables required; strikethrough/task lists if cheap.
- Two aesthetic grades: barebones and classy (tasteful, not maximalist).
- Auto-reload the page when the file/dir changes.
- Preserve scroll position across reloads.

Avoid feature creep beyond this list.

Closed: Implemented, twice codex-reviewed, verified with curl in both modes. Accepted tradeoffs documented in docs/log.md.
