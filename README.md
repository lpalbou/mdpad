# mdpad

A fast, beautiful markdown reader and editor for the terminal.

`less` and `more` show markdown as raw markup: headers drown in `#`s, nested
bullets collapse, and tables become pipe soup. `mdpad` renders the document
properly — styled headings, aligned tables that adapt to your terminal width,
syntax-highlighted code, nested lists and quotes — and lets you edit the file
without leaving the viewer.

## Highlights

- **Tables that actually work.** Column widths adapt to the terminal using a
  staged layout algorithm: typical content and headers are protected first,
  and only the space-hungry columns wrap. A 10-column benchmark table stays
  readable at 80 columns.
- **Numeric columns auto right-align**, the way a human would lay them out.
- **Full navigation**: vim/less keys, incremental search with highlighting,
  a table-of-contents jump panel, scrollbar, mouse wheel.
- **Editing built in**: press `e` for a built-in editor (undo/redo, save with
  `Ctrl+S`, instant re-render), or `E` to use your `$EDITOR`.
- **Behaves in pipes**: `mdpad README.md | grep …` prints clean text;
  `--print` renders ANSI for terminals; respects `NO_COLOR`.
- **Single static binary.** No runtime, no config files, instant startup.
  Works on Linux, macOS and Windows.

## Install

From source (needs a Rust toolchain):

```bash
cargo install --path .
```

Prebuilt binaries for Linux (x86_64/aarch64, static musl), macOS
(Intel/Apple Silicon) and Windows are produced by the release workflow and
attached to GitHub releases.

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
| `m` | toggle mouse capture (off = native text selection/copy) |
| `e` | edit in built-in editor (`Ctrl+S` save, `Esc` back) |
| `E` | edit in `$EDITOR`, reload on exit |
| `r` | reload file from disk |
| `?` | help |
| `q` | quit |

### Options

| Flag | Effect |
|---|---|
| `-p`, `--print` | render to stdout instead of opening the viewer |
| `-w`, `--width <N>` | render width (default: terminal width) |
| `--color <auto\|always\|never>` | color policy for print mode |
| `--light` | light theme |
| `--ascii` | ASCII-only glyphs |
| `--urls` | always show link URLs inline |
| `--prose-width <N>` | cap prose line length (default 100; 0 = full width) |
| `--no-highlight` | disable code syntax highlighting |
| `--no-mouse` | keep native terminal text selection |

## Notes

- When stdout is piped, `mdpad` prints instead of opening the viewer, and
  strips colors unless `--color always` is given.
- Editing is disabled for stdin documents (there is no file to save to).
- On terminals without truecolor, syntax highlighting quantizes to 256 colors.

## Roadmap

- Horizontal table scrolling with a frozen first column for tables that
  cannot fit even after degradation (the current cascade keeps them readable,
  but very wide tables deserve first-class panning).
- Editor soft-wrap: migrate to `ratatui-textarea` (ratatui 0.30) — the
  current widget scrolls long prose lines horizontally.
- `y` to copy a code block / table (as TSV or markdown) without border
  pollution.
- OSC 8 hyperlinks in print mode.
- Lazy per-block layout for very large (multi-MB) documents.

## Development

```bash
cargo test        # unit + integration tests
cargo run -- tests/fixtures/showcase.md
```

See `docs/Overview.md` for architecture and `docs/DataFlow.md` for the
rendering pipeline.

## License

MIT
