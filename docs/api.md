# CLI & keys

`mdpad` is a CLI tool; this page is its canonical interface reference:
command line, keys, environment variables and exit behavior. New here?
Start with [Getting started](getting-started.md).

## Synopsis

```text
mdpad [OPTIONS] [FILE]
```

`FILE` is a markdown file, or `-` to read from stdin. With no argument,
mdpad reads stdin when it is piped. When stdout is not a terminal (a pipe or
redirect), mdpad prints the rendered document instead of opening the viewer.

## Options

| Flag | Effect |
|---|---|
| `-p`, `--print` | render to stdout instead of opening the viewer |
| `-w`, `--width <N>` | render width in columns (default: terminal width; minimum 20) |
| `--color <auto\|always\|never>` | color policy; `auto` strips colors when stdout is piped (default: `auto`) |
| `--light` | light theme (default: dark) |
| `--ascii` | ASCII-only glyphs, no box drawing or unicode bullets |
| `--urls` | always show link URLs inline after the link text |
| `--prose-width <N>` | cap prose line length for readability (default 100; 0 = full width) |
| `--no-highlight` | disable code syntax highlighting |
| `--no-mouse` | never capture the mouse (keeps terminal-native selection) |
| `-h`, `--help` | print help |
| `-V`, `--version` | print version |

## Viewer keys

| Key | Action |
|---|---|
| `j` / `k`, `↓` / `↑` | scroll one line (`Enter` also scrolls down) |
| `Space` / `b`, `PgDn` / `PgUp`, `f` | scroll one page |
| `d` / `u` | scroll half page |
| `g` / `G`, `Home` / `End` | go to top / bottom |
| `/` | incremental search; `Enter` confirms, `Esc` cancels |
| `n` / `N` | next / previous search match |
| `t` | table of contents (`j`/`k` move, `Enter` jumps, `Esc` closes) |
| `L` | show/hide link URLs inline |
| mouse click on a link | follow it: local files open in the viewer, `#anchors` jump to their heading, URLs open in the browser/OS handler |
| `Backspace` | go back to the document you followed a link from |
| mouse wheel | scroll (3 lines per notch) |
| mouse drag | select text; copied to clipboard on release ([details](clipboard.md)) |
| `Ctrl+C` | copy the current selection again; without a selection, quit |
| `Esc` | clear search and selection / close overlay |
| `m` | toggle mouse capture (off = terminal-native selection) |
| `e` | open the built-in editor |
| `E` | open `$VISUAL`/`$EDITOR`, reload on exit |
| `r` | reload the file from disk |
| `?` | help overlay |
| `q` | quit |

Link following needs mouse capture (on by default; `m` toggles). Relative
link targets resolve against the current document's directory — against the
working directory for stdin documents. The back history is kept in memory,
so `Backspace` returns to a piped document too. Destinations with a URI
scheme (`https:`, `mailto:`, …) are handed to the OS handler.

Mermaid code blocks get a `view in browser` link on their label line:
clicking it opens the diagram rendered in the mermaid.live viewer (the
source travels in the URL fragment and is not sent to any server). See the
[FAQ](faq.md#can-mdpad-render-mermaid-diagrams) for why diagrams are not
drawn in the terminal.

## Editor keys

| Key | Action |
|---|---|
| `Ctrl+S` | save (atomic: temp file + rename) and re-render |
| `Esc` | back to viewer (asks to save/discard/cancel if unsaved) |
| `Ctrl+Z` / `Ctrl+Y` | undo / redo |

Editing is disabled for stdin documents (there is no file to save to).
CRLF line endings, missing final newlines and UTF-8 BOMs round-trip
byte-identical through an open/save cycle; files with mixed line endings
warn that saving unifies them.

## Environment variables

| Variable | Effect |
|---|---|
| `NO_COLOR` | disables all color output (viewer and print mode) |
| `COLORTERM` | when it advertises `truecolor`/`24bit`, syntax highlighting uses RGB; otherwise colors quantize to the 256-color palette |
| `COLUMNS` | render width when output is piped and no `--width` is given |
| `VISUAL`, `EDITOR` | external editor for `E` (checked in that order; may contain arguments, runs through the shell) |

## Exit behavior

- Exit code 0 on success, 1 on errors (unreadable file, no input); errors
  print to stderr as `mdpad: <message>`.
- Broken pipes in print mode (`mdpad x.md | head`) exit cleanly.
- The terminal is always restored on exit — including panics and
  termination signals (`SIGTERM`/`SIGHUP`/`SIGINT` quit like `q`) — so a
  crash or a `kill` never leaves your shell in raw mode.
- If the terminal itself disappears (emulator crash, orphaned pty), the
  viewer notices the hangup and exits immediately instead of lingering.

## See also

- [Selection & clipboard](clipboard.md) — copy semantics and terminal support
- [Troubleshooting](troubleshooting.md) — when a key or flag does not do
  what you expect
