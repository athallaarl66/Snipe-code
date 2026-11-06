## Purpose

Defines stable command outcomes for scripts and CI so invalid audit use, execution failures, unsupported exports, and policy-level findings are distinguishable from a successful audit.

## ADDED Requirements

### Requirement: CLI uses stable process exit codes
The CLI SHALL use exit code `0` for successful commands with no policy failure, `1` for completed audit findings that violate the configured failure threshold, `2` for invalid usage or unsupported options, and `3` for runtime, filesystem, parsing, or export failures. Interactive cancellation SHALL exit `0` when no failure occurred and SHALL leave state unchanged.

#### Scenario: Successful audit
- **WHEN** audit completes and no finding meets the failure threshold
- **THEN** the process exits with code `0` after requested reports are written

#### Scenario: Policy-level finding
- **WHEN** audit completes with one or more findings at or above the configured failure threshold
- **THEN** requested reports are still written and the process exits with code `1`

#### Scenario: Invalid export format
- **WHEN** `--export` contains an unsupported or empty format token
- **THEN** the process writes a usage error to standard error and exits with code `2` before audit or report generation

#### Scenario: Audit execution failure
- **WHEN** the audit target cannot be scanned or a requested report cannot be written
- **THEN** the process writes the failure to standard error and exits with code `3`

#### Scenario: User cancels overwrite
- **WHEN** the user cancels project generation after a collision warning
- **THEN** no files change and the process exits with code `0`

### Requirement: Audit export accepts only supported formats
Audit export parsing SHALL accept a non-empty comma-separated set of unique supported formats and SHALL reject empty tokens, unknown tokens, and unsupported combinations without silently ignoring them.

#### Scenario: Multiple supported formats
- **WHEN** the user requests `md,pdf,docx`
- **THEN** all three requested report formats are generated

#### Scenario: Mixed supported and unknown formats
- **WHEN** the user requests `pdf,csv`
- **THEN** the command rejects the request with code `2` and identifies `csv` as unsupported

#### Scenario: Repeated format
- **WHEN** the user requests `md,md`
- **THEN** the command either normalizes the duplicate before generation or rejects it clearly; it SHALL NOT generate conflicting repeated writes silently

### Requirement: Audit exposes a machine-readable result summary
The audit library boundary SHALL return the generated report paths together with counts by severity, highest severity, detected scopes, and the policy failure decision needed by the CLI. The CLI SHALL derive its process code from that result rather than discarding findings after export.

#### Scenario: Findings remain available after export
- **WHEN** reports are generated for a project containing findings
- **THEN** the returned result preserves severity counts and determines the non-zero exit code

#### Scenario: No findings
- **WHEN** evaluated rules produce no findings
- **THEN** the result reports zero counts and does not fail solely because the result is empty

### Requirement: Failures use standard error
Operational errors, usage errors, and policy failures SHALL be written to standard error. Standard output SHALL be reserved for requested command results and report paths.

#### Scenario: Runtime error
- **WHEN** report writing fails
- **THEN** standard error contains the failure and standard output does not claim audit completion
