# Usage & keys

## Invocation

```bash
mdpad FILE.md                  # interactive viewer
mdpad -                        # read from stdin
curl -s https://example.com/doc.md | mdpad
mdpad --print FILE.md          # render to stdout (ANSI)
mdpad -p -w 80 FILE.md         # fixed-width render
mdpad --light FILE.md          # light terminal background
mdpad --ascii FILE.md          # no box-drawing/unicode glyphs
mdpad -p FILE.md | grep …      # pipes get clean text automatically
```

## Keys (viewer)

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
| `e` | edit in built-in editor |
| `E` | edit in `$EDITOR`, reload on exit |
| `r` | reload file from disk |
| `?` | help overlay |
| `q` | quit |

## Keys (editor)

| Key | Action |
|---|---|
| `Ctrl+S` | save (atomic: temp file + rename) |
| `Esc` | back to viewer (asks if unsaved) |
| `Ctrl+Z` / `Ctrl+Y` | undo / redo |

Editing is disabled for stdin documents (there is no file to save to).
CRLF line endings, missing final newlines and UTF-8 BOMs round-trip
byte-identical through an open/save cycle.

## Options

| Flag | Effect |
|---|---|
| `-p`, `--print` | render to stdout instead of opening the viewer |
| `-w`, `--width <N>` | render width (default: terminal width; minimum 20) |
| `--color <auto\|always\|never>` | color policy (`auto` strips colors when piped) |
| `--light` | light theme |
| `--ascii` | ASCII-only glyphs |
| `--urls` | always show link URLs inline |
| `--prose-width <N>` | cap prose line length (default 100; 0 = full width) |
| `--no-highlight` | disable code syntax highlighting |
| `--no-mouse` | never capture the mouse |

`NO_COLOR` is respected everywhere, in both the viewer and print mode.
