# Rule 10: Rust Standards, Code Quality & Error Handling

## Language & Compiler Standards
- **Rust Edition**: 2021 Edition.
- **Minimum Supported Rust Version (MSRV)**: 1.80.0+.
- **Zero Warnings**: Code must always compile cleanly under `cargo clippy --workspace --all-targets -- -D warnings`.
- **Formatting**: Standard formatting strictly enforced via `cargo fmt --all -- --check`.

## Error Handling Philosophy
1. **Libraries & Domain Crates (`cay-core`, `cay-providers`)**:
   - Use `thiserror` for typed, well-defined error enums.
   - Every public trait method must return `Result<T, ProviderError>` or Option types where failure is expected.
   - **Strictly prohibit `unwrap()` or `expect()` in providers**: WinRT APIs frequently fail due to permissions, missing services, or transient OS state. Errors must be gracefully captured, logged, and propagated.
2. **Application Crate (`cay-app`)**:
   - Use `anyhow::Result` and `anyhow::Context` for high-level application setup, configuration parsing, and startup orchestration.
   - Log errors using `tracing` or `eprintln!` and fall back to safe default states (e.g., Idle pill) rather than crashing the process.

## Performance & Memory Guidelines
- Minimize dynamic heap allocations inside per-frame view renderers.
- Use static strings or small stack-allocated arrays where appropriate.
- Ensure background worker threads sleep or park when no events occur to maintain near 0% idle CPU usage.
