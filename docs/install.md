# Install

## Prebuilt binaries (recommended)

Grab the archive for your platform from the
[latest release](https://github.com/lpalbou/mdpad/releases/latest),
unpack, and put `mdpad` on your `PATH`:

```bash
# Linux x86_64 (fully static, works on any distro)
curl -sL https://github.com/lpalbou/mdpad/releases/download/v0.1.0/mdpad-v0.1.0-x86_64-unknown-linux-musl.tar.gz | tar xz
sudo mv mdpad /usr/local/bin/

# macOS (Apple Silicon)
curl -sL https://github.com/lpalbou/mdpad/releases/download/v0.1.0/mdpad-v0.1.0-aarch64-apple-darwin.tar.gz | tar xz
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

Every asset ships with a `.sha256` checksum
(`sha256sum -c mdpad-*.tar.gz.sha256`).

On macOS, Gatekeeper may quarantine the unsigned binary; clear it with
`xattr -d com.apple.quarantine mdpad`. On Windows, SmartScreen may warn the
first time — the checksums are the integrity guarantee.

## From crates.io

```bash
cargo install mdpad
```

## From source

```bash
git clone https://github.com/lpalbou/mdpad
cd mdpad
cargo install --path . --locked
```

Requires a stable Rust toolchain (1.88+). No C compiler or system libraries
needed — the dependency tree is pure Rust.
