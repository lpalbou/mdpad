# FAQ

Common questions about mdpad. For symptom-driven fixes, see
[Troubleshooting](troubleshooting.md).

## Why doesn't `Cmd+C` copy my selection on macOS?

macOS terminals handle `Cmd+C` themselves (it copies the terminal's own
native selection) and never forward the keystroke to the application
running inside. That is why mdpad copies **on mouse release** instead —
select, release, paste. `Ctrl+C` re-copies the current selection. Details:
[Selection & clipboard](clipboard.md).

## Does copying work over SSH?

Yes, in terminals that support the OSC 52 clipboard escape (Kitty, WezTerm,
Ghostty, Alacritty, iTerm2 with its clipboard preference enabled, Windows
Terminal, tmux ≥ 3.3). The copy lands on your local clipboard even though
mdpad runs on the remote machine. See the
[support matrix](clipboard.md#how-the-copy-travels).

## Why did my selection copy table borders and bullets?

Mouse selection is screen-faithful: you copy what you see, exactly like
selecting in the terminal itself. For clean raw markdown, press `e` and
copy from the built-in editor. A semantic copy key (`y`, copying the block
under the cursor without decoration) is on the roadmap.

## Why does a wide table render as a list of `Header: value` records?

Below a certain width a grid physically cannot hold the content (imagine
15 columns in 20 cells). Rather than overflow or shear, mdpad degrades the
table to per-row records that keep every cell readable. Give it more width
(or use `--print -w N` and a pager that scrolls horizontally) to get the
grid back. The algorithm: [Rendering pipeline](rendering.md#table-sizing).

## Why can't I edit a document I piped in?

There is no file to save to. Editing needs a real path; stdin documents
open read-only (the built-in editor tells you so). Save the stream to a
file first if you want to edit it.

## Where is the config file?

There is none, by design. mdpad is a single static binary with sensible
defaults; behavior is controlled by flags (`--light`, `--ascii`,
`--prose-width`, …) and standard environment variables (`NO_COLOR`,
`VISUAL`/`EDITOR`). See [CLI & keys](api.md).

## Does mdpad respect `NO_COLOR`?

Yes, everywhere — both the interactive viewer and print mode. `--color
never` does the same per-invocation; `--color always` keeps colors when
piping. Bold and italic survive colorless mode (they are typography, not
color).

## Which markdown dialect is supported?

CommonMark plus the GitHub extensions: tables, strikethrough, task lists
and footnotes (via pulldown-cmark). Raw HTML blocks are shown as dimmed
source text, not interpreted.

## Why are there no images?

Terminals have no portable inline-image protocol. Images render as their
alt text with an image marker. Protocol-specific rendering (Kitty/iTerm2
graphics) is out of scope for now.

## How do I get plain text out of mdpad?

Pipe it: `mdpad doc.md | grep …` strips all styling automatically. Or
`mdpad --print --color never doc.md > doc.txt`.

## See also

- [Getting started](getting-started.md)
- [CLI & keys](api.md)
- [Troubleshooting](troubleshooting.md)
