# Repository Guidelines

## Project Structure & Module Organization
This repository is a single Rust crate (`voxine`) defined in `Cargo.toml`.
- Core library entrypoint: `src/lib.rs`
- Engine and runtime modules: `src/engine.rs`, `src/task*.rs`, `src/worker.rs`, `src/mpsc.rs`
- Domain modules: `src/chunk/`, `src/world_gen/`, `src/physics/`, `src/netcode/`
- Experimental/legacy code: `src/deprecated/`
- Performance benchmarks: `benches/*.rs` (Criterion)

Keep new modules small and focused; expose public APIs through `src/lib.rs` re-exports.

## Build, Test, and Development Commands
Use Cargo for all local workflows:
- `cargo check` - fast type and borrow checking during development.
- `cargo build` - full debug build.
- `cargo test` - runs unit tests (`#[cfg(test)]`, currently centered in `src/test.rs`).
- `cargo bench` - runs Criterion benchmarks in `benches/`.
- `cargo fmt` - formats code with Rustfmt.
- `cargo clippy --all-targets --all-features -D warnings` - lint gate before PRs.

## Coding Style & Naming Conventions
- Follow standard Rust style (Rustfmt, 4-space indentation, trailing commas in multiline literals).
- Use `snake_case` for functions/modules/files, `CamelCase` for types/traits, `SCREAMING_SNAKE_CASE` for constants.
- Prefer explicit, descriptive names (`ChunkID`, `RenderThreadChannels`) over abbreviations.
- Keep module visibility minimal (`mod` by default, `pub` only when needed).

## Testing Guidelines
- Add unit tests next to the relevant module or in `src/test.rs` for cross-module behavior.
- Name tests by behavior, e.g. `block_coord_floors_negative_values`.
- For performance-sensitive changes (meshing, chunk formats, channels), add or update Criterion benches under `benches/`.
- Run `cargo test` and, when performance is impacted, `cargo bench` before opening a PR.

## Commit & Pull Request Guidelines
Git history uses short, descriptive, imperative-style subjects (for example: `Update Mesh and Meshing`, `Changed Mesh sending API`).
- Commit title: <=72 chars, clear scope, one logical change.
- PRs should include: purpose, key design/behavior changes, and validation steps run (`cargo test`, `cargo bench`, etc.).
- Link related issues/tasks when applicable; include benchmark deltas for performance-related PRs.

## Security & Configuration Tips
- Do not commit secrets or machine-specific settings.
- Keep dependency changes intentional; review `Cargo.lock` diffs before merge.
