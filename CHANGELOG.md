# Changelog

## 0.6.0 (unreleased)

### Added

- Interactive launcher and task-specific AI agent workflows.
- Opt-in local scan history a33s
  Checking time-core v0.1.2
  Checking jpeg-decoder v0.3.2
  Checking color_quant v1.1.0
  Checking gif v0.13.3
  Checking time v0.3.36
  Checking tiff v0.9.1
  Checking png v0.17.16
  Checking hashbrown v0.14.5
  Compiling serde_derive v1.0.229
  Checking pom v3.4.0
  Checking block-buffer v0.10.4
  Checking crypto-common v0.1.7
  Checking encoding_rs v0.8.35
  Checking ttf-parser v0.19.2
  Checking log v0.4.33
  Checking fastrand v2.5.0
  Checking linked-hash-map v0.5.6
  Checking bytemuck v1.25.2
  Checking md5 v0.7.0
  Checking unicode-width v0.2.2
  Checking lopdf v0.31.0
  Checking console v0.15.11
  Checking image v0.24.9
  Checking tempfile v3.27.0
  Checking owned_ttf_parser v0.19.0
  Checking digest v0.10.7
  Checking hashlink v0.9.1
  Checking zip v0.6.6
  Compiling include_dir_macros v0.7.4
  Checking cpufeatures v0.2.17
  Checking fallible-streaming-iterator v0.1.9
  Checking xml-rs v0.8.29
  Checking zeroize v1.9.0
  Checking shell-words v1.1.1
  Checking smallvec v1.16.5
  Checking fallible-iterator v0.3.0
  Checking base64 v0.13.1
  Checking dialoguer v0.11.0
  Checking include_dir v0.7.4
  Checking docx-rs v0.4.15
  Checking sha2 v0.10.9
  Checking printpdf v0.7.0
  Checking uuid v1.26.1
  Checking rusqlite v0.32.1
  Checking snipe-code v0.5.3 (/home/runner/work/Snipe-code/Snipe-code)
  error: couldn't read `tests/../docs/features/quality-gates.md`: No such file or directory (os error 2)
  --> tests/quality_integration_test.rs:368:16
  |
  368 | let docs = include_str!("../docs/features/quality-gates.md").replace("\r\n", "\n");
  | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  |
  = note: this error originates in the macro `include_str` (in Nightly builds, run with -Z macro-backtrace for more info)

error: could not compile `snipe-code` (test "quality_integration_test") due to 1 previous error
warning: build failed, waiting for other jobs to finish...
Error: Process completed with exit code 101.
nd versioned scan results.

- Rust SAST candidates, offline npm advisory analysis, and quality metrics with baseline comparisons.
- Read-only QA planning and Postman collection drafts.

### Changed

- Verification previews gate effects and requires approval for project commands. Unavailable and blocked gates no longer count as a complete pass.
- `scan` reads without writing reports; `export` writes selected report formats to an explicit directory. Legacy `audit` still writes reports in the scanned directory.
- Generated-project verification limits automatic approval to its own temporary workspace and clears recognized inherited database target variables.
- Recognized database credentials are filtered from verification output within the limits described in the README. Review logs before sharing; unknown and transformed secrets can still appear.

### Fixed

- Reject unsafe database targets before install or test gates and distinguish missing configuration from a safe target.
- Reject linked audit and template entries that escape their root, with canonical-path checks. These checks are not atomic against concurrent filesystem changes.
- Preserve report content across PDF pages and align generated Compose database configuration.

### Verification limits

- POSIX symlink regression tests were not run locally. Windows junction coverage passed; Windows file symlink coverage was skipped when the required privilege was unavailable.
- Temporary workspaces do not sandbox project commands or prevent network access.

## 0.5.3

Previous published release. See the GitHub release for its notes and artifacts.
