# mdpad

A fast, beautiful markdown reader and editor for the terminal.

`less` and `more` show markdown as raw markup: headers drown in `#`s, nested
bullets collapse, and tables become pipe soup. `mdpad` renders the document
properly — styled headings, aligned tables that adapt to your terminal width,
syntax-highlighted code, nested lists and quotes — and lets you select text,
copy it, and edit the file without leaving the viewer.

## Why mdpad

- **Tables that actually work.** Column widths adapt to the terminal using a
  staged layout algorithm: typical content and headers are protected first,
  and only the space-hungry columns wrap. A 10-column benchmark table stays
  readable at 80 columns; tables that cannot physically fit as a grid degrade
  to a clean per-row record view instead of overflowing.
- **Numeric columns auto right-align**, the way a human would lay them out.
- **Select and copy with the mouse.** Drag to select rendered text; it lands
  on your system clipboard the moment you release the button — locally and
  over SSH.
- **Full navigation**: vim/less keys, incremental search with highlighting,
  a table-of-contents jump panel, scrollbar, mouse wheel.
- **Editing built in**: a built-in editor with atomic saves and instant
  re-render, or hand off to your `$EDITOR`.
- **Behaves in pipes**: pipe to `grep` and get clean text; `--print` renders
  ANSI for terminals; respects `NO_COLOR`.
- **Single static binary.** No runtime, no config files, instant startup.
  Works on Linux, macOS and Windows.

## Quick start

```bash
mdpad README.md               # interactive viewer
mdpad --print README.md       # render to stdout
curl -s https://example.com/doc.md | mdpad
```

Press `?` inside the viewer for the complete key reference, `q` to quit.

Continue with [Getting started](getting-started.md), or jump straight to
[CLI & keys](api.md) for the full reference and
[Selection & clipboard](clipboard.md) for how copying works.
