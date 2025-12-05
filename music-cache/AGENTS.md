# Repository Guidelines

## Project Structure & Modules
- `src/`: core library; `db/` holds sled-backed storage helpers, `library_scan.rs` for filesystem traversal, `music_metadata.rs` for tag parsing, `ffi.rs` for C FFI surface, `tests/` for unit helpers.
- `music-cache-derive/`: proc-macro crate generating boilerplate for the main library.
- `tests/`: Rust integration tests plus `ffi_shim.c` used by the build script for C-side validation.
- `music_cache.h`: exported C header for embedders; keep in sync with `ffi.rs` changes.
- `target/` and `tmp_cache/`: build artifacts and scratch data; do not commit.

## Build, Test, and Development Commands
- When making any code changes always verify all of the following commands are green and formatters/linters are run.
- `cargo build` — compile the library and proc-macro crates.
- `cargo test --features integration-tests` — run unit and integration tests; build.rs compiles `tests/ffi_shim.c` automatically.
- `cargo fmt` — format Rust sources.
- `find tests -name '*.c' -print0 | xargs -0 clang-format -i` — format all C sources (LLVM preset in .clang-format).
- `cargo clippy --all-targets --all-features` — lint with warnings treated seriously; fix or justify any new warnings.

## Coding Style & Naming Conventions
- Follow `rustfmt` defaults; prefer small, focused modules and functions.
- Naming: `snake_case` for functions/files/modules, `UpperCamelCase` for types/traits, `SCREAMING_SNAKE_CASE` for constants.
- Error handling: bubble errors with context; avoid `unwrap`/`expect` outside tests unless justified.
- Public surface: keep FFI-safe types and signatures aligned with `music_cache.h`; document any ABI changes.
- Code should be terse, but elegant and readable. Self explanatory without need for comments.

## Testing Guidelines
- Framework: standard Rust tests plus C shim-driven FFI checks in `tests/ffi_c_test.rs`.
- Arbitrary generators live in `src/tests/common.rs`, these should be used where possible.
- All code changes should either be covered by an existing test or have a new test.
- Add unit tests near the code (`src/.../tests`), and integration tests under `tests/`.
- Prefer deterministic fixtures; when using temp data, rely on `tempfile` helpers.
- Aim to keep coverage of new code meaningful; exercise FFI boundaries when changing `ffi.rs`.

## Commit & Pull Request Guidelines
- Commits: short, imperative summaries (e.g., “add ffi cache lookup”); keep related changes together.
- PRs: include scope summary, linked issue/ticket, and test results (commands run and outcomes). Attach screenshots or logs if touching FFI behavior or external consumers.
- Keep diffs minimal; note any feature flags or config toggles that affect behavior (`integration-tests`).

## FFI & Safety Notes
- Any change to `ffi.rs` must be mirrored in `music_cache.h` and be tested to ensure shim builds.
- Maintain `repr(C)` correctness and avoid panics across the FFI boundary
