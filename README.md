# mdpad

A fast, beautiful markdown reader and editor for the terminal.

`less` and `more` show markdown as raw markup: headers drown in `#`s, nested
bullets collapse, and tables become pipe soup. `mdpad` renders the document
properly — styled headings, aligned tables that adapt to your terminal width,
syntax-highlighted code, nested lists and quotes — and lets you select text,
copy it, and edit the file without leaving the viewer.

## Highlights

- **Tables that actually work.** Column widths adapt to the terminal using a
  staged layout algorithm: typical content and headers are protected first,
  and only the space-hungry columns wrap. A 10-column benchmark table stays
  readable at 80 columns; tables that cannot fit as a grid degrade to a clean
  per-row record view instead of overflowing.
- **Numeric columns auto right-align**, the way a human would lay them out.
- **Select and copy with the mouse.** Drag to select rendered text; it is
  copied to your system clipboard the moment you release the button — locally
  and over SSH. See [Selection & clipboard](docs/clipboard.md).
- **Full navigation**: vim/less keys, incremental search with highlighting,
  a table-of-contents jump panel, scrollbar, mouse wheel.
- **Links you can follow.** Click a link: local markdown files open in the
  viewer (`Backspace` goes back), `#anchors` jump to their heading, and
  external URLs open in your browser. Mermaid blocks carry a one-click
  `view in browser` link that opens the rendered diagram on mermaid.live.
- **Editing built in**: press `e` for a built-in editor (undo/redo, atomic
  save with `Ctrl+S`, instant re-render), or `E` to use your `$EDITOR`.
- **Behaves in pipes**: `mdpad README.md | grep …` prints clean text;
  `--print` renders ANSI for terminals; respects `NO_COLOR`.
- **Single static binary.** No runtime, no config files, instant startup.
  Works on Linux, macOS and Windows.

## Install

Prebuilt binaries for Linux (x86_64/aarch64, static musl), macOS
(Intel/Apple Silicon) and Windows are attached to
[GitHub releases](https://github.com/lpalbou/mdpad/releases/latest).

```bash
cargo install mdpad          # from crates.io
cargo install --path .       # from a source checkout
```

See [Getting started](docs/getting-started.md) for per-platform
instructions and checksums.

## Usage

```bash
mdpad README.md              # interactive viewer
mdpad -                      # read from stdin (pipe)
curl -s https://example.com/doc.md | mdpad
mdpad --print README.md      # render to stdout (ANSI)
mdpad -p -w 80 README.md     # fixed width render
mdpad --light README.md      # light terminal background
mdpad --ascii README.md      # no box-drawing/unicode glyphs
```

### Keys

| Key | Action |
|---|---|
| `j` / `k`, arrows | scroll line |
| `Space` / `b`, `PgDn` / `PgUp` | scroll page |
| `d` / `u` | scroll half page |
| `g` / `G`, `Home` / `End` | top / bottom |
| `/` then `n` / `N` | incremental search, next/previous match |
| `t` | table of contents (Enter jumps) |
| `L` | show/hide link URLs |
| mouse click | follow link (local file / `#anchor` / browser) |
| `Backspace` | back to the previous document |
| mouse drag | select text; copied to clipboard on release (`Esc` clears) |
| `Ctrl+C` | copy selection again (without a selection: quit) |
| `m` | toggle mouse capture (off = terminal-native selection) |
| `e` | edit in built-in editor (`Ctrl+S` save, `Esc` back) |
| `E` | edit in `$EDITOR`, reload on exit |
| `r` | reload file from disk |
| `?` | help |
| `q` | quit |

The complete key and flag reference lives in
[CLI & keys](docs/api.md).

## Documentation

The full documentation is published at
[lpalbou.github.io/mdpad](https://lpalbou.github.io/mdpad) and lives in
[`docs/`](docs/README.md):

- [Getting started](docs/getting-started.md) — install, first run, first edit
- [CLI & keys](docs/api.md) — every flag, key and environment variable
- [Selection & clipboard](docs/clipboard.md) — how copy works, terminal support
- [Architecture](docs/architecture.md) — components, invariants, design shape
- [Rendering pipeline](docs/rendering.md) — data flow and the table layout algorithm
- [FAQ](docs/faq.md) — common questions and limitations
- [Troubleshooting](docs/troubleshooting.md) — symptoms, causes, fixes

AI tools can start from [`llms.txt`](llms.txt) and
[`llms-full.txt`](llms-full.txt).

## Roadmap

- Semantic copy (`y`): copy the code block or table under the cursor as
  clean text/TSV/markdown, without border glyphs.
- Horizontal table scrolling with a frozen first column for tables that
  cannot fit even after degradation.
- Editor soft-wrap (the current widget scrolls long prose lines
  horizontally).
- OSC 8 hyperlinks in print mode.
- Lazy per-block layout for very large (multi-MB) documents.

## Contributing & policies

- [CONTRIBUTING.md](CONTRIBUTING.md) — development setup, tests, PR workflow
- [SECURITY.md](SECURITY.md) — how to report vulnerabilities
- [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) — community expectations
- [CHANGELOG.md](CHANGELOG.md) — release history
- [ACKNOWLEDGEMENTS.md](ACKNOWLEDGEMENTS.md) — upstream projects and prior art

## License

[MIT](LICENSE)
