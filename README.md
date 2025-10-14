# fdb

Rust workspace for a Linux x86-64 debugger.

## Workspace Layout

- `crates/libfdb`: Core library exposing debugger primitives shared by other crates.
- `crates/fdb`: CLI frontend built on top of the library.
- `tests/`: Integration harness executed via `cargo test`.
- `.github/workflows/ci.yml`: Minimal CI runner executing formatting, linting, builds, and tests.

## Build & Test (Arch Linux)

```bash
sudo pacman -S --needed base-devel rustup
rustup default stable
cargo fmt -- --check
cargo check
cargo test
cargo run -p fdb -- --help
```
