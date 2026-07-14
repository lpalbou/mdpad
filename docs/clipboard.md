# Selection & clipboard

mdpad lets you select rendered text with the mouse and puts it on your
system clipboard — locally and over SSH. This page explains the exact
behavior, why it works this way, and which terminals support what.

## How to use it

1. Drag with the left mouse button over the rendered document. The
   selection highlights in reverse video and follows document lines, so you
   can keep scrolling (wheel) mid-drag to extend it.
2. Release the button. The selection is copied to the clipboard immediately
   ("copy on select", as in tmux or PuTTY) and the status bar confirms with
   `copied N lines to clipboard`.
3. Paste anywhere with `Cmd+V` / `Ctrl+V`.

`Ctrl+C` re-copies the current selection. `Esc` or any click dismisses it.
Without a selection, `Ctrl+C` keeps its usual meaning: quit.

You copy what you see: table borders, list bullets and heading marks are
part of the rendered text and travel with the copy. Trailing padding
(e.g. code-block background fill) is trimmed automatically. To copy the
*raw markdown* instead, press `e` and select inside the editor, or use
`mdpad --print`.

## Why copy-on-release instead of Cmd+C

macOS terminals handle `Cmd+C` themselves — it copies the *terminal's*
native selection and is never forwarded to the application running inside.
No terminal application can bind it. Copying on mouse release gives every
platform the same zero-keystroke flow: select, then paste.

## How the copy travels

Every copy is sent through two complementary channels:

1. **Native OS clipboard** — NSPasteboard on macOS, the Win32 clipboard on
   Windows, X11/Wayland selections on Linux. Reliable for local sessions,
   including terminals with no escape-sequence clipboard support.
2. **OSC 52 escape sequence** — asks the terminal emulator itself to set
   the clipboard. This is what makes copying work **over SSH**, where the
   clipboard you want to reach lives on your local machine.

Terminal support for OSC 52:

| Terminal | OSC 52 |
|---|---|
| Kitty, WezTerm, Ghostty, Alacritty | yes |
| iTerm2 | yes — enable *Settings → General → Selection → "Applications in terminal may access clipboard"* |
| Windows Terminal | yes |
| tmux | ≥ 3.3 (`set-clipboard` defaults to on) |
| macOS Terminal.app | no — local copies still work via the native channel |
| legacy Windows console (conhost) | no — local copies still work via the native channel |

On X11 without a clipboard manager, a copy made by mdpad remains available
while mdpad is running; the OSC 52 channel (owned by the terminal) persists
beyond that.

## Prefer your terminal's own selection?

Press `m` to release the mouse: mdpad stops capturing mouse events, wheel
scrolling stops, and your terminal's native click-drag selection and
copy shortcuts work as usual. Press `m` again to re-capture. Start with
`--no-mouse` to never capture the mouse.

## See also

- [CLI & keys](api.md) — the full key reference
- [Troubleshooting](troubleshooting.md#copy-does-not-paste) — when a copy
  does not arrive in the clipboard
