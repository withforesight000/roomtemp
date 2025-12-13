# src-tauri tests and coverage

Run Rust unit tests:

```bash
cd src-tauri
cargo test
```

Install `cargo-llvm-cov` (if missing):

```bash
cargo install cargo-llvm-cov
```

Run coverage report (produces `lcov.info`):

```bash
cd src-tauri
cargo llvm-cov --workspace --lcov --output-path lcov.info
```

Notes:
- Tests use `mockall` for mocking where appropriate.
- The `lcov.info` can be uploaded to your coverage service or inspected with `genhtml`/`lcov` tools.
