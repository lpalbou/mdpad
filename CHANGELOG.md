# Changelog

## 0.1.0 — unreleased

Initial version.

### Fixed (adversarial review hardening, pre-release)

- Infinite loop: a double-width grapheme (CJK/emoji) at wrap width 1 —
  reachable via deeply nested quotes on narrow terminals — spun forever;
  unbreakable single graphemes now emit on their own line.
- Stack overflow: 20k-deep quote/list nesting overflowed the stack (SIGSEGV
  skips the panic hook, leaving the shell in raw mode). Model depth is now
  capped at 64; deeper structure flattens into the parent container.
- Editor data safety: saves are now atomic (temp file + rename, preserving
  permissions and writing through symlinks) so a crash or full disk cannot
  destroy the original; a file containing only "\n" no longer becomes empty
  on save; mixed CRLF/LF files warn that saving unifies endings.
- Terminal state: a failure between entering raw mode and completing TUI
  setup now restores the terminal instead of leaving the shell broken.
- Ctrl+C in the unsaved-changes dialog no longer silently discards edits.
- NO_COLOR / --color never now apply to the interactive viewer, not just
  print mode.
- Smart-case search translates match offsets through the case fold, fixing
  dropped matches and wrong highlights on lines with length-changing folds
  (Turkish İ, Kelvin sign); undo back to the saved state clears the dirty
  flag; searches survive resize without teleporting to the first match.
- Tables that cannot physically fit as a grid (e.g. 15 columns at width 20)
  now degrade to a per-row record layout instead of overflowing.
- Wrapped H3+ headings no longer repeat their `###` prefix on every line;
  checked-task glyph no longer uses an emoji-block character that misaligns
  columns in many fonts; tabs in code expand to real tab stops.
- Event loop: input bursts (paste, resize storms) coalesce into one redraw;
  idle no longer redraws 4×/s; match highlighting no longer scans all
  matches per visible row.
- Release workflow: cross pinned, Windows checksums sha256sum-compatible,
  tag-vs-Cargo.toml version gate, musl target built in CI.
- Width invariant: tables with tiny cells no longer overflow the terminal
  (a fake minimum budget let a 9-column table emit 37-wide lines at width
  20); tables that would squeeze below word width now render as records
  instead of vertical confetti; CJK cells can no longer shear borders; long
  code-fence language labels truncate.
- Styling: spaces inside styled runs (inline code background, link
  underline, strikethrough) keep the run's style instead of rendering holes.
- Ordered task-list items keep their numbers ("2. ✔ ship it") and mixed
  lists share one aligned marker field.
- Multi-line HTML blocks render contiguously with indentation preserved
  (previously one double-spaced block per source line).
- Header-only tables no longer draw a phantom empty row (grid) or vanish
  (records); date-like columns (2026-07-13) no longer right-align.
- Colorless output marks inline code with backticks — previously
  indistinguishable from prose.

### Added

- Markdown rendering engine: semantic block model over pulldown-cmark
  (CommonMark + GFM tables, strikethrough, task lists, footnotes).
- Width-adaptive table layout with staged degradation (protect headers and
  typical content first, drop padding second, wrap header words third,
  hard-break giant tokens last) and max-min fair column sizing.
  Rationale: tables are the single worst-rendered construct in every
  existing terminal pager; this is the core feature.
- Numeric-column detection: undeclared alignments right-align when ≥ 70% of
  body cells are numeric, matching how humans format tables.
- Style-preserving word wrap with correct CJK/emoji display widths and
  grapheme-safe hard breaks for URLs/identifiers.
- Syntax highlighting for fenced code blocks (syntect, pure-Rust build),
  with 256-color quantization for terminals without truecolor.
- Interactive viewer: less/vim navigation, incremental smart-case search
  with match highlighting, table-of-contents jump panel, scrollbar, mouse
  wheel, resize re-flow, status bar, help overlay.
- Built-in editor (tui-textarea): undo/redo, Ctrl+S save with instant
  re-render, dirty-state confirm dialog; `E` opens `$VISUAL`/`$EDITOR` and
  reloads on exit.
- Print mode for pipes/scripts (`--print`, auto-detected on redirected
  stdout) with `--color auto|always|never` and `NO_COLOR` support.
- Themes: dark (default), light, plain; `--ascii` glyph set for terminals
  without box-drawing fonts.
- Terminal safety: panic hook restores the terminal state so a crash never
  leaves the shell in raw mode.
- Test suite: 35 tests covering wrapping (incl. CJK), table sizing, parser
  shapes, search, ANSI output, plus binary-level integration tests asserting
  no-overflow and content preservation across widths 30–200 and pathological
  inputs.
