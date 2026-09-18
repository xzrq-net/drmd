# Mermaid

`mermaid.min.js` is the unmodified standalone browser bundle from
[Mermaid 12.0.0](https://github.com/mermaid-js/mermaid/releases/tag/mermaid%4012.0.0).
It is embedded in the Rust binary and loaded only for Mermaid code blocks.
No CDN or JavaScript build step is needed.

Source archive: <https://registry.npmjs.org/mermaid/-/mermaid-12.0.0.tgz>

- Bundle: `package/dist/mermaid.min.js`
- License: `package/LICENSE`, saved as `mermaid.LICENSE`; bundled dependency
  notices are also preserved in the JavaScript file.
- Bundle SHA-256: `28fca7ae6ebc7ed7bb63bde63136a74bfef14f296a57e403657eeb8b32836073`

Updates are demand-driven: a needed syntax feature, a rendering bug, browser
compatibility, or a relevant security fix in Mermaid or its bundled dependencies.
Cargo and flake lockfile updates do not update this copy. Review release notes
before upgrading: even established diagrams can change appearance (v12 changed
the default layout and theme).

To update, download the chosen version's npm archive, verify its integrity
against the npm registry metadata, replace these two files, and update the
version and checksum here. Use the standalone bundle: the ESM distribution
requires additional chunks. Check valid and invalid diagrams in a browser,
including live reload, with external network requests blocked.
