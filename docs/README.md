# mdpad documentation

mdpad is a fast, beautiful markdown reader and editor for the terminal.
This folder is the complete documentation set; it is also published as a
book at [lpalbou.github.io/mdpad](https://lpalbou.github.io/mdpad).

New to mdpad? Read [Getting started](getting-started.md) first.

## User guide

| Page | What it covers |
|---|---|
| [Introduction](introduction.md) | What mdpad is and why it exists |
| [Getting started](getting-started.md) | Install (binaries, crates.io, source), first run, first edit |
| [CLI & keys](api.md) | Every flag, viewer/editor key, environment variable, exit behavior |
| [Selection & clipboard](clipboard.md) | Mouse selection, copy-on-release, OSC 52 + native clipboard, terminal support |
| [FAQ](faq.md) | Recurring questions, supported markdown dialect, known limitations |
| [Troubleshooting](troubleshooting.md) | Symptom → cause → fix: clipboard, colors, tables, terminal state |

## Internals

| Page | What it covers |
|---|---|
| [Architecture](architecture.md) | Components, invariants, design decisions, mode state machine |
| [Rendering pipeline](rendering.md) | Data flow and the staged table-sizing algorithm |

## Project

- [Changelog](changelog.md) — release history (includes the root
  [CHANGELOG.md](../CHANGELOG.md))
- [CONTRIBUTING.md](../CONTRIBUTING.md) — development setup, tests, PR workflow
- [SECURITY.md](../SECURITY.md) — vulnerability reporting
- [CODE_OF_CONDUCT.md](../CODE_OF_CONDUCT.md) — community expectations
- [ACKNOWLEDGEMENTS.md](../ACKNOWLEDGEMENTS.md) — upstream projects and prior art

AI tools can start from the repository root's [`llms.txt`](../llms.txt)
(concise index) and [`llms-full.txt`](../llms-full.txt) (full corpus).

The book is built with [mdBook](https://rust-lang.github.io/mdBook/);
[`SUMMARY.md`](SUMMARY.md) defines its navigation.
