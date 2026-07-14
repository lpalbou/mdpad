# Acknowledgements

mdpad stands on excellent open-source foundations.

## Core dependencies

- [pulldown-cmark](https://github.com/pulldown-cmark/pulldown-cmark) —
  CommonMark + GFM parsing; battle-tested handling of markdown edge cases.
- [ratatui](https://github.com/ratatui/ratatui) — the terminal UI
  framework, with [crossterm](https://github.com/crossterm-rs/crossterm)
  providing one event/terminal API across Linux, macOS and Windows.
- [tui-textarea](https://github.com/rhysd/tui-textarea) — the editor
  widget with undo/redo.
- [syntect](https://github.com/trishume/syntect) — syntax highlighting
  for fenced code blocks (pure-Rust regex build).
- [arboard](https://github.com/1Password/arboard) — native clipboard
  access on macOS, Windows, X11 and Wayland.
- [unicode-width](https://github.com/unicode-rs/unicode-width) and
  [unicode-segmentation](https://github.com/unicode-rs/unicode-segmentation)
  — correct display widths and grapheme boundaries for CJK/emoji.
- [clap](https://github.com/clap-rs/clap) — CLI parsing;
  [base64](https://github.com/marshallpierce/rust-base64) — OSC 52 payload
  encoding.

## Prior art and inspiration

- `less` and `more` — the pager ergonomics mdpad borrows (keys, incremental
  search) while replacing raw markup with real rendering.
- [glow](https://github.com/charmbracelet/glow) and
  [mdcat](https://github.com/swsnr/mdcat) — earlier terminal markdown
  renderers that shaped expectations; mdpad focuses on width-adaptive
  tables and in-place editing.
- The [OSC 52](https://invisible-island.net/xterm/ctlseqs/ctlseqs.html)
  convention and the tmux/PuTTY copy-on-select idiom, which make terminal
  clipboard integration possible.

## Tooling

- [mdBook](https://rust-lang.github.io/mdBook/) with
  [mdbook-mermaid](https://github.com/badboy/mdbook-mermaid) for the
  documentation site.
- [cross](https://github.com/cross-rs/cross) for static musl release
  builds.
