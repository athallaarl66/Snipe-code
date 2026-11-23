# Changelog

## 0.6.1

### Added

- `snipe-code skill [agent] [--install]` previews or installs a global `snipe-code` agent skill for claude, opencode, codex, hermes, cursor, github-copilot, windsurf, antigravity, and gemini-cli. Existing skill directories are never overwritten.
- The interactive launcher shows an ASCII banner with the compiled version and author, and adds an "Install global AI skill" entry.

### Changed

- README install guidance names the built executable path, the PowerShell call operator, and a first manual check on a disposable project.

### Fixed

- Release quality gate installs `pdftotext` before running the test suite, so the PDF report parity tests stop failing on the runner.

## 0.6.0

### Added

- Interactive launcher and task-specific AI agent workflows.
- Opt-in local scan history and versioned scan results.
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
