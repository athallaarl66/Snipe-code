## Purpose

Defines deterministic verification reports that can be consumed by CI systems and reviewed by people without requiring an AI service.

## ADDED Requirements

### Requirement: Verification exports structured and human-readable reports
Verification SHALL support JSON, JUnit XML, and Markdown report formats that represent the same parts, gates, commands, states, details, summary counts, and overall result shown by the terminal report.

#### Scenario: JSON report
- **WHEN** a user requests a JSON verification report
- **THEN** the output is valid JSON containing the target, per-part gate outcomes, summary counts, and overall result

#### Scenario: JUnit report
- **WHEN** a user requests a JUnit XML verification report
- **THEN** the output is valid JUnit-compatible XML in which failed, unavailable, and skipped gates remain distinguishable

#### Scenario: Markdown report
- **WHEN** a user requests a Markdown verification report
- **THEN** the output contains the target, each project part, gate outcome, executed command, failure detail, and summary

### Requirement: Report output is explicit and safe
A requested verification report SHALL be written only to the explicit output path supplied by the user, and invalid format or output combinations SHALL fail before any gate runs.

#### Scenario: Explicit destination
- **WHEN** the user supplies a supported report format and output file
- **THEN** the CLI writes that report to the named destination and identifies the written path

#### Scenario: Missing output destination
- **WHEN** a file report is requested without an output destination
- **THEN** the CLI reports a usage error and runs no gate

#### Scenario: Unsupported report format
- **WHEN** a user requests PDF, DOCX, HTML, or an unknown verification report format
- **THEN** the CLI reports the supported formats and runs no gate

### Requirement: Report generation does not change verification policy
The report renderer SHALL consume the deterministic verification result and SHALL NOT use AI, reinterpret gate output, or change the process exit status determined by verification.

#### Scenario: Failed gate with successful report write
- **WHEN** a gate fails and the requested report is written successfully
- **THEN** the report records the failure and the process exits with verification failure status `1`

#### Scenario: Report write failure
- **WHEN** gates finish but the requested report cannot be written
- **THEN** the CLI reports the export failure on standard error and exits with runtime status `3`

### Requirement: Incomplete results remain non-pass after export
Successful report writes SHALL preserve gate-derived exit 0, 1, 3, or 4 and exact blocked, unavailable, and not-applicable states. Usage errors SHALL remain exit 2; report write failures SHALL return runtime exit 3.

#### Scenario: Applicable gate blocked or unavailable
- **WHEN** a report is exported for no-execution or incomplete verification
- **THEN** output preserves exact gate states and exits 3 or 4 respectively instead of success; JUnit skipped cases include exact-state properties

### Requirement: Security audit and verification reports remain distinct
User-facing output SHALL identify audit exports as security reports and verification exports as test and quality-gate reports.

#### Scenario: Audit export
- **WHEN** `snipe-code audit` writes Markdown, PDF, DOCX, or JSON output
- **THEN** no output describes those files as test reports

#### Scenario: Verification export
- **WHEN** `snipe-code verify` writes JSON, JUnit XML, or Markdown
- **THEN** no output describes the report as a security audit
