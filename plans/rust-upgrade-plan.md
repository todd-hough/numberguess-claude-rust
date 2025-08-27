# Rust and Dependencies Upgrade Plan

## Overview
This document outlines the comprehensive plan for upgrading Rust to the latest stable edition and updating all library dependencies for the number_guessing_game project.

## Current State
- **Rust Version**: 1.89.0 (2025-08-04)
- **Edition**: 2021
- **Last Updated**: Check git history for dependency updates

## Upgrade Tasks

### 1. Check and Upgrade Rust Toolchain to Latest Stable

#### 1.1 Check Current Rust Stable Version
```bash
rustup check
```
- Verify what the latest stable version is available

#### 1.2 Update Rustup Itself
```bash
rustup self update
```
- Ensures rustup tool is latest version before updating Rust

#### 1.3 Update to Latest Stable
```bash
rustup update stable
```
- Downloads and installs the latest stable Rust toolchain

#### 1.4 Verify New Rust Version
```bash
rustc --version
```
- Confirm the upgrade was successful

### 2. Check for Outdated Dependencies

#### 2.1 Install cargo-outdated Tool
```bash
cargo install cargo-outdated
```
- Only needed if not already installed
- Provides detailed information about available updates

#### 2.2 Run Dependency Analysis
```bash
cargo outdated
```
- Lists all dependencies with available updates
- Shows current version, latest version, and compatibility

### 3. Update Dependencies in Cargo.toml

#### 3.1 Update rand
- Current: 0.9
- Check latest: `cargo search rand`
- Update in Cargo.toml to latest version

#### 3.2 Update clap
- Current: 4.5
- Check latest: `cargo search clap`
- Update in Cargo.toml to latest version
- Maintain "derive" feature

#### 3.3 Update axum
- Current: 0.8
- Check latest: `cargo search axum`
- Update in Cargo.toml to latest version
- Review breaking changes in changelog

#### 3.4 Update tokio
- Current: 1
- Check latest: `cargo search tokio`
- Update in Cargo.toml to latest version
- Maintain "full" feature set

#### 3.5 Update serde and serde_json
- Current: serde 1.0, serde_json 1.0
- Check latest versions
- Update both to maintain compatibility
- Keep "derive" feature for serde

#### 3.6 Update tower and tower-http
- Current: tower 0.5, tower-http 0.6
- Check latest versions
- Update both for compatibility
- Maintain tower-http features: ["cors", "fs"]

#### 3.7 Update reqwest (dev dependency)
- Current: 0.12
- Check latest: `cargo search reqwest`
- Update in Cargo.toml
- Maintain "json" feature

### 4. Update Cargo.lock and Verify Compatibility

#### 4.1 Update Lock File
```bash
cargo update
```
- Updates Cargo.lock with new dependency versions
- Resolves dependency tree

#### 4.2 Verify Compilation
```bash
cargo check
```
- Ensures no compilation errors with new dependencies
- Quick verification without full build

### 5. Check for Rust Edition Upgrade

#### 5.1 Check Edition 2024 Availability
```bash
cargo --version
```
- Verify if edition 2024 is available and stable
- Check Rust blog/release notes for edition status

#### 5.2 Update Edition in Cargo.toml
- If edition 2024 is stable, change:
  ```toml
  edition = "2024"
  ```

#### 5.3 Run Edition Migration
```bash
cargo fix --edition
```
- Automatically updates code for new edition idioms
- Apply suggested fixes

### 6. Run Comprehensive Tests and Checks

#### 6.1 Release Build Test
```bash
cargo build --release
```
- Ensures optimized build works with all updates
- Catches any release-specific issues

#### 6.2 Run Test Suite
```bash
cargo test
```
- Verifies all tests pass with updated dependencies
- Ensures functionality is preserved

#### 6.3 Run Clippy Lints
```bash
cargo clippy -- -W clippy::all
```
- Checks for code quality issues
- Identifies deprecated patterns

#### 6.4 Format Code
```bash
cargo fmt
```
- Ensures consistent code formatting
- Apply any new formatting rules

### 7. Review and Document Breaking Changes

#### 7.1 Review Dependency Changelogs
- Check each updated dependency's changelog/release notes
- Identify breaking changes that affect the codebase
- Key areas to review:
  - axum API changes
  - clap derive macro changes
  - tokio runtime changes
  - tower middleware updates

#### 7.2 Update Code for Breaking Changes
- Fix any compilation errors from API changes
- Update deprecated function calls
- Refactor code to use new patterns/idioms

#### 7.3 Update Documentation
- Add upgrade notes to CLAUDE.md
- Document any significant changes
- Note new minimum Rust version requirements

## Rollback Plan

If issues arise during upgrade:

1. **Revert Cargo.toml changes**
   ```bash
   git checkout -- Cargo.toml
   ```

2. **Restore Cargo.lock**
   ```bash
   git checkout -- Cargo.lock
   ```

3. **Downgrade Rust if needed**
   ```bash
   rustup default <previous-version>
   ```

## Success Criteria

- [ ] All dependencies updated to latest stable versions
- [ ] Code compiles without warnings
- [ ] All tests pass
- [ ] Clippy shows no significant issues
- [ ] Application runs correctly (manual testing)
- [ ] Web server endpoints function properly
- [ ] No performance regressions

## Notes

- Always review breaking changes before updating major versions
- Test thoroughly in development before deploying
- Consider updating dependencies incrementally if many breaking changes exist
- Keep the old Cargo.lock as backup until upgrade is verified stable