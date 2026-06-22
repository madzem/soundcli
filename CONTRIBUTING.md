# Contributing to soundcli

Thanks for your interest in improving soundcli! This is a small project, so the
process is light.

## Getting started

soundcli is a Linux-only Rust binary. You'll need:

- A recent stable Rust toolchain — install via [rustup](https://rustup.rs).
- `libdbus-1-dev` and `pkg-config` (for the `mpris` dependency).
- At runtime: a Chromium-family browser, PipeWire's `wpctl` (for volume), and a
  D-Bus session — none of which are needed just to build and test.

```bash
git clone https://github.com/madzem/soundcli
cd soundcli
cargo build
cargo run -- --demo   # try the UI with sample data, no browser needed
```

## Before opening a pull request

CI runs the same four checks; please run them locally first:

```bash
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
cargo build
cargo test
```

## Guidelines

- **Formatting & lints**: code must be `rustfmt`-clean and pass Clippy with no
  warnings. `unsafe` code is forbidden (`unsafe_code = "forbid"`).
- **Tests**: add unit tests for pure logic (parsing, URL handling, config). The
  browser/MPRIS/PipeWire paths can't be unit-tested in CI — verify those manually
  and describe how in the PR.
- **Commits**: keep them focused, with a clear message explaining the *why*.
- **Comments**: explain rationale (`//`), not the obvious. Public items get `///`.

## Reporting bugs

Open an issue with your distro, browser, and the output of:

```bash
soundcli --list-players
SOUNDCLI_DEBUG=1 soundcli --dump-queue "<url>"
```
