# Getting started

This page takes you from nothing to reading, searching and editing markdown
in the terminal. For the complete flag and key reference, see
[CLI & keys](api.md).

## Install

### Prebuilt binaries (recommended)

Download the archive for your platform from the
[latest release](https://github.com/lpalbou/mdpad/releases/latest), unpack,
and put `mdpad` on your `PATH`:

```bash
# Linux x86_64 (fully static, works on any distro)
curl -sL https://github.com/lpalbou/mdpad/releases/download/v0.2.0/mdpad-v0.2.0-x86_64-unknown-linux-musl.tar.gz | tar xz
sudo mv mdpad /usr/local/bin/

# macOS (Apple Silicon)
curl -sL https://github.com/lpalbou/mdpad/releases/download/v0.2.0/mdpad-v0.2.0-aarch64-apple-darwin.tar.gz | tar xz
sudo mv mdpad /usr/local/bin/
```

Available targets:

| Platform | Archive |
|---|---|
| Linux x86_64 (static musl) | `mdpad-v*-x86_64-unknown-linux-musl.tar.gz` |
| Linux aarch64 (static musl) | `mdpad-v*-aarch64-unknown-linux-musl.tar.gz` |
| macOS Intel | `mdpad-v*-x86_64-apple-darwin.tar.gz` |
| macOS Apple Silicon | `mdpad-v*-aarch64-apple-darwin.tar.gz` |
| Windows x86_64 | `mdpad-v*-x86_64-pc-windows-msvc.zip` |

Every asset ships with a `.sha256` checksum; verify with
`sha256sum -c mdpad-*.tar.gz.sha256`.

On macOS, Gatekeeper may quarantine the unsigned binary; clear it with
`xattr -d com.apple.quarantine mdpad`. On Windows, SmartScreen may warn the
first time — the checksums are the integrity guarantee.

### From crates.io

```bash
cargo install mdpad
```

### From source

```bash
git clone https://github.com/lpalbou/mdpad
cd mdpad
cargo install --path . --locked
```

Requires a stable Rust toolchain (1.88+). No C compiler or system libraries
needed — the dependency tree is pure Rust.

## First run

Open any markdown file:

```bash
mdpad README.md
```

You get the rendered document: styled headings, aligned tables, highlighted
code. From there:

1. **Scroll** with the mouse wheel, `j`/`k`, `Space`, or `d`/`u`.
2. **Search** with `/` — matches highlight as you type; `n`/`N` jump between
   them.
3. **Jump by heading** with `t`: a table-of-contents panel opens, `Enter`
   jumps to the selected section.
4. **Copy something**: drag with the mouse to select rendered text — it is
   copied to your clipboard when you release the button. Paste it anywhere.
   Details and terminal support: [Selection & clipboard](clipboard.md).
5. **Edit** with `e`: a built-in editor opens on the raw markdown. `Ctrl+S`
   saves (atomically) and re-renders instantly; `Esc` returns to the viewer.
   Prefer your own editor? `E` opens `$VISUAL`/`$EDITOR` and reloads on exit.
6. Press `?` anytime for the full key reference, `q` to quit.

## Pipes and scripts

`mdpad` detects redirected output and prints instead of opening the viewer:

```bash
mdpad README.md | grep -A2 "Install"   # clean text, no escapes
mdpad --print README.md                # ANSI render to a terminal
mdpad -p -w 80 --color always doc.md   # fixed width, keep colors
curl -s https://example.com/doc.md | mdpad   # read from stdin
```

`NO_COLOR` is respected everywhere. See [CLI & keys](api.md) for all flags.

## If something looks wrong

Copy not pasting, colors off, tables squeezed — see
[Troubleshooting](troubleshooting.md).
