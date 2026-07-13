# Overview

## Goal

A terminal markdown reader/editor whose whole reason to exist is **rendering
quality and navigation**: headings, nested/mixed lists and above all tables
must stay readable at any terminal width. Secondary goal: quick edits without
leaving the tool. Constraints: OS-agnostic (Linux/macOS/Windows), single
lightweight binary, zero configuration.

## Why Rust

- Single static binary (musl on Linux): nothing to install but the file.
- `crossterm` gives one event/terminal API across all three OSes, including
  the Windows console.
- `pulldown-cmark` is a strict CommonMark + GFM parser, so parsing edge cases
  are outsourced to a battle-tested library.
- `ratatui` + `tui-textarea` cover the TUI and the editor widget.

## Core components

| Component | Path | Responsibility |
|---|---|---|
| Model | `src/markdown/model.rs` | Semantic document tree (blocks + inlines), independent of any rendering concern |
| Parser | `src/markdown/parser.rs` | pulldown-cmark event stream -> model, tolerant of malformed input |
| Theme | `src/render/theme.rs` | All colors/glyphs; dark, light and plain (no-color) variants; unicode + ascii charsets |
| Wrap | `src/render/wrap.rs` | Style-preserving word wrap, CJK/emoji aware, grapheme-safe hard breaks |
| Inline | `src/render/inline.rs` | Inlines -> styled spans; link URL policy |
| Block | `src/render/block.rs` | Blocks -> pre-wrapped visual lines; lists, quotes, code, headings |
| Table | `src/render/table.rs` | Column sizing (staged water-fill), alignment heuristics, borders |
| Highlight | `src/render/highlight.rs` | syntect code highlighting, 256-color quantization |
| ANSI | `src/render/ansi.rs` | Rendered lines -> ANSI text for print mode |
| App | `src/app.rs` | State machine (view/search/toc/help/edit/confirm) + event loop |
| UI | `src/ui/*` | Viewer, search, TOC, help, editor, terminal lifecycle |

## Key design decisions

1. **Pre-wrapped lines.** The renderer emits lines that each occupy exactly
   one terminal row. Scrolling, search jumps, TOC anchors and the scrollbar
   are then exact index arithmetic — no estimation, no drift. Resize simply
   re-renders at the new width.
2. **One renderer, two frontends.** The TUI viewer and `--print` share the
   same rendering pipeline, so print output is a faithful, testable proxy for
   what the viewer shows. Integration tests assert invariants (no line
   overflows the width, no content lost) across many widths.
3. **Tables degrade in stages** (see DataFlow.md): natural widths -> protect
   headers+typical content -> drop padding -> wrap headers at words -> only
   then hard-break words. A single outlier cell cannot reserve air for a
   whole column (80th-percentile "typical width" rule).
4. **Editing is raw-text, reader-first.** The built-in editor edits markdown
   source with undo/redo and save-and-re-render; `$EDITOR` integration covers
   power users. No WYSIWYG: that scope kills tools of this size.
5. **Trust the terminal.** 256-color indexed palette by default, truecolor
   only for syntax highlighting when `COLORTERM` advertises it; `--ascii` for
   glyph-poor environments; `NO_COLOR` honored.

## Testing strategy

- Unit tests per module (wrap correctness incl. CJK, table sizing, parser
  shapes, search smart-case, ANSI escapes).
- Integration tests run the real binary over fixtures (including the
  benchmark report that motivated the project) at widths 30..200 and assert:
  no overflow, all content present, styled lines self-terminate, pathological
  inputs exit cleanly.
- TUI smoke-tested via a pty (enter, scroll, TOC, editor, quit, restore).
