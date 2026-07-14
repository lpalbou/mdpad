# Troubleshooting

Symptom-oriented fixes. If your question is conceptual rather than broken,
try the [FAQ](faq.md) first.

## Copy does not paste

You dragged a selection, saw `copied N lines to clipboard`, but `Cmd+V` /
`Ctrl+V` pastes something else.

1. **Local session in macOS Terminal.app or legacy Windows console** —
   these ignore the OSC 52 escape, but the native clipboard channel should
   still work. If it does not, your build may predate v0.2.0: check
   `mdpad --version`.
2. **iTerm2** — enable *Settings → General → Selection → "Applications in
   terminal may access clipboard"*. Without it, iTerm2 drops OSC 52 writes
   (local copies still arrive through the native channel).
3. **Over SSH** — only the OSC 52 channel can reach your local clipboard,
   so the terminal on *your* side must support it: Kitty, WezTerm, Ghostty,
   Alacritty, iTerm2 (preference above), Windows Terminal. See the
   [support matrix](clipboard.md#how-the-copy-travels).
4. **Inside tmux** — needs tmux ≥ 3.3 and `set-clipboard` on (the
   default). Check with `tmux show -g set-clipboard`.
5. **X11, after quitting mdpad** — without a clipboard manager, the native
   X11 copy belongs to the mdpad process and disappears when it exits.
   Paste while mdpad runs, or rely on a terminal with OSC 52 support.

## Mouse selection selects the whole screen / wrong text

Your terminal is selecting natively because mouse capture is off. Press
`m` to re-enable capture (the status bar confirms), or drop `--no-mouse`
from your invocation. The two modes are exclusive by nature: capture on =
mdpad selection + wheel scrolling; capture off = terminal-native selection.

## Wheel scrolling stopped working

Same cause as above: mouse capture is off (`m` toggles it back on).
Keyboard scrolling (`j`/`k`, `Space`, `d`/`u`) always works.

## My shell is broken after a crash (no echo, weird characters)

mdpad restores the terminal even on panics, but if the process was killed
with `SIGKILL` nothing can run cleanup. Type `reset` (or `stty sane`) and
press Enter to restore the shell. Please report the crash with the
document that triggered it: [security & bug reports](../SECURITY.md).

## Colors look wrong or washed out

- Syntax highlighting quantizes to 256 colors unless `COLORTERM` contains
  `truecolor` or `24bit` — most modern terminals set this themselves.
- On a light background, run with `--light`; the default theme assumes
  dark.
- `NO_COLOR` set in your environment disables all color; unset it or pass
  nothing (there is no override flag for enabling color in the viewer).
- Piped output strips color by design; use `--color always` to keep ANSI
  codes (`mdpad -p --color always doc.md | less -R`).

## Box-drawing characters render as garbage

Your terminal font or locale lacks the glyphs. Run with `--ascii` for
pure-ASCII borders and bullets. If characters *misalign* instead (borders
shear in tables with CJK or emoji), please report it — cell-width
correctness is a core goal.

## A table is squeezed into `Header: value` records

Expected at very narrow widths — the grid physically cannot fit. Widen the
terminal or render with a larger fixed width: `mdpad -p -w 120 doc.md`.
Rationale and the exact algorithm: [Rendering pipeline](rendering.md#table-sizing).

## `E` says `$EDITOR is not set`

Set one of `VISUAL` or `EDITOR` (checked in that order), e.g.
`export EDITOR=vim` in your shell profile. Values with arguments work
(`export EDITOR="code --wait"`). On Windows, notepad is used as fallback.

## macOS: "cannot be opened because the developer cannot be verified"

The release binaries are not code-signed. Clear the quarantine attribute:
`xattr -d com.apple.quarantine mdpad`. Verify the download first with the
`.sha256` checksum next to each release asset.

## Saving says the file has mixed line endings

The file contains both CRLF and LF endings; saving unifies them (a warning
tells you before you save). Uniform files — CRLF or LF, with or without a
final newline — round-trip byte-identical.

## Still stuck?

Open an issue at
[github.com/lpalbou/mdpad/issues](https://github.com/lpalbou/mdpad/issues)
with your terminal, OS, `mdpad --version`, and if possible the document
(or a minimized sample) that reproduces the problem.
