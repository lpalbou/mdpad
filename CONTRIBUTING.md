# Contributing to mdpad

Thanks for your interest in improving mdpad. This guide covers local setup,
testing, and what a good pull request looks like.

## Development setup

You need a stable Rust toolchain (1.88+) — nothing else. The dependency
tree is pure Rust; no C compiler or system libraries.

```bash
git clone https://github.com/lpalbou/mdpad
cd mdpad
cargo test                              # unit + integration tests
cargo run -- tests/fixtures/showcase.md # try the viewer on the stress fixture
```

`tests/fixtures/showcase.md` exercises every construct the renderer must
handle; `tests/fixtures/report.md` is the wide-table document that motivated
the project.

## Before you open a PR

CI enforces all of these; running them locally saves a round trip:

```bash
cargo test --all-targets --locked
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
```

CI also builds a static `x86_64-unknown-linux-musl` binary and runs
narrow-width smoke renders, so keep changes portable across Linux, macOS
and Windows.

## Code orientation

Read [docs/architecture.md](docs/architecture.md) first — it maps every
module and states the invariants (most importantly: one rendered line is
exactly one terminal row). [docs/rendering.md](docs/rendering.md) explains
the table sizing algorithm.

Guidelines that keep the codebase healthy:

- Prefer general-purpose logic over special cases; tests describe examples,
  the code must handle the whole input space.
- Keep files small and focused; comments explain *why*, not what.
- Integration tests assert properties (no overflow, no content loss), not
  exact bytes — new rendering features should follow that pattern.

## Documentation

User-facing docs live in [`docs/`](docs/README.md) and are published with
mdBook. If your change affects behavior, update the relevant page and
[CHANGELOG.md](CHANGELOG.md) in the same PR, and regenerate `llms.txt` /
`llms-full.txt` if page structure changed.

To preview the book locally:

```bash
cargo install mdbook mdbook-mermaid
mdbook-mermaid install .
mdbook serve
```

## Releases

Releases are tag-driven: pushing `vX.Y.Z` (matching `Cargo.toml`) builds
binaries for five targets, creates the GitHub release with checksums, and
publishes to crates.io via trusted publishing. Maintainers handle this;
contributors only need to keep `CHANGELOG.md` accurate.

## Reporting issues

- Bugs: [github.com/lpalbou/mdpad/issues](https://github.com/lpalbou/mdpad/issues)
  — include OS, terminal, `mdpad --version` and a minimal reproducing
  document when possible.
- Security issues: see [SECURITY.md](SECURITY.md) — please report privately.

By participating you agree to the [Code of Conduct](CODE_OF_CONDUCT.md).
