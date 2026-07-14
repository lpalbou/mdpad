# Changelog

## 0.2.0 — 2026-07-15

### Added

- Mouse text selection in the viewer: drag to select rendered text; the
  selection highlights in reverse video, is anchored to document lines (it
  survives scrolling, and the wheel extends it mid-drag), and is copied to
  the system clipboard the moment the button is released. `Ctrl+C`
  re-copies the current selection and still quits when there is none;
  `Esc` or any click dismisses it. Cell mapping is CJK/emoji-aware, and
  trailing padding is trimmed from copied lines.
- Clipboard integration through two channels: the native OS clipboard
  (macOS NSPasteboard, Win32, X11/Wayland) and the OSC 52 escape sequence,
  so copying works both locally — including terminals without OSC 52
  support — and over SSH. See
  [docs/clipboard.md](docs/clipboard.md) for the terminal support matrix.

### Changed

- The `m` key still toggles mouse capture; with capture on, dragging now
  selects inside mdpad instead of doing nothing. Terminal-native selection
  remains available with capture off (or `--no-mouse`).
- Documentation set rebuilt: new user guide
  ([getting started](docs/getting-started.md), [CLI & keys](docs/api.md),
  [selection & clipboard](docs/clipboard.md), [FAQ](docs/faq.md),
  [troubleshooting](docs/troubleshooting.md)), internals pages with
  architecture diagrams, contributor/security/conduct policies, and
  AI-readable `llms.txt` / `llms-full.txt` indexes.

## 0.1.0 — 2026-07-13

Initial release.

- Markdown rendering engine: semantic block model over pulldown-cmark
  (CommonMark + GFM tables, strikethrough, task lists, footnotes),
  tolerant of malformed and adversarial input (depth-capped nesting,
  no-overflow width invariant, grapheme-safe wrapping).
- Width-adaptive table layout with staged degradation (protect headers and
  typical content first, drop padding second, wrap header words third,
  hard-break giant tokens last), max-min fair column sizing, and a per-row
  record fallback for tables that cannot fit as a grid.
- Numeric-column detection: undeclared alignments right-align when ≥ 70%
  of body cells are numeric.
- Style-preserving word wrap with correct CJK/emoji display widths and
  grapheme-safe hard breaks for URLs/identifiers.
- Syntax highlighting for fenced code blocks (syntect), with 256-color
  quantization for terminals without truecolor.
- Interactive viewer: less/vim navigation, incremental smart-case search
  with match highlighting, table-of-contents jump panel, scrollbar, mouse
  wheel, resize re-flow, status bar, help overlay.
- Built-in editor (tui-textarea): undo/redo, atomic `Ctrl+S` save with
  instant re-render, dirty-state confirm dialog; `E` opens
  `$VISUAL`/`$EDITOR` and reloads on exit. CRLF endings, missing final
  newlines and UTF-8 BOMs round-trip byte-identical.
- Print mode for pipes/scripts (`--print`, auto-detected on redirected
  stdout) with `--color auto|always|never` and `NO_COLOR` support.
- Themes: dark (default), light, plain; `--ascii` glyph set for terminals
  without box-drawing fonts.
- Terminal safety: panic hook restores the terminal state so a crash never
  leaves the shell in raw mode.
- Release automation: prebuilt binaries for Linux (x86_64/aarch64, static
  musl), macOS (Intel/Apple Silicon) and Windows, with SHA-256 checksums;
  published to crates.io.
