# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

* Permalinks for each diagnostic

## 0.3.0 - 06-13-2024

### Changed

* Migrated `wdl-gauntlet` to use the new parser implementation ([#76](https://github.com/stjude-rust-labs/wdl/pull/76))

## 0.2.0 - 05-31-2024

### Changed

* Core goal of crate is split in two:
  * **The goal of** (base) **`wdl-gauntlet` is to ensure the parsing of syntactically valid WDLs never regresses.**
  * **The goal of `wdl-gauntlet --arena` is to test lint rules against WDL "in the wild".**
* `LintWarnings` are ignored (when there is no `--arena` flag)
* uses `libgit2` (via the `git2` crate) instead of the GitHub REST API (via `octocrab` and `reqwest` crates)
* no more persistent cache (Now uses `temp-dir`)

### Added

* The `--arena` flag and `Arena.toml` for lint rule testing
* more test repos!
* test repos are tracked at specific commits

## 0.1.0 — 12-17-2023

### Added

* Adds the initial version of the crate.
