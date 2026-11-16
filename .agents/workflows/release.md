# Workflow: Release & Packaging

This workflow specifies the procedure for cutting a production or pre-release build of Cay.

## Pre-Release Checks
Before starting a release:
1. Ensure all tests and lints pass:
   ```bash
   cargo test --workspace
   cargo clippy --workspace --all-targets -- -D warnings
   cargo fmt --all -- --check
   ```
2. Verify all state snapshots headlessly via `.agents/workflows/verify-ui.md`.
3. Confirm git working tree is clean.

---

## Release Procedure

### Step 1: Version Bump
1. Bump the `version` field across all workspace crates:
   - Root `Cargo.toml` (if workspace-versioned) or individual crate `Cargo.toml` files:
     - `crates/cay-core/Cargo.toml`
     - `crates/cay-providers/Cargo.toml`
     - `crates/cay-app/Cargo.toml`
2. Follow Semantic Versioning (`MAJOR.MINOR.PATCH`).

### Step 2: Update Changelog
1. Open `CHANGELOG.md`.
2. Group changes under standard headings:
   - `Added` for new features or states.
   - `Changed` for changes in existing functionality or design tokens.
   - `Fixed` for bug fixes.
   - `Security` for security enhancements.

### Step 3: Build Optimized Release Binaries
Compile with full link-time optimization (LTO):
```powershell
cargo build --release -p cay-app
```
Binary output: `target/release/cay-app.exe`.

### Step 4: Code Signing (Authenticode)
Sign the executable using a valid Windows Code Signing Certificate:
```powershell
signtool sign /fd SHA256 /a /tr http://timestamp.digicert.com /td SHA256 target/release/cay-app.exe
```

### Step 5: Packaging (Installer / MSIX)
- If packaged as standard standalone installer (Inno Setup / WiX):
  - Compile the installer script into `Cay-Setup-vX.Y.Z.exe`.
  - Sign the installer `.exe`.
- If packaged as sparse MSIX (for UserNotificationListener identity):
  - Run `MakeAppx pack /d <manifest_dir> /p Cay-vX.Y.Z.msix`.
  - Sign the `.msix` package.

### Step 6: Git Tag & GitHub Release
1. Create signed git tag:
   ```bash
   git tag -s vX.Y.Z -m "Release vX.Y.Z"
   git push origin vX.Y.Z
   ```
2. Publish release notes and binary assets to GitHub Releases.
