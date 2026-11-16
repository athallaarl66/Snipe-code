# Spec Delta

## Purpose

Defines how the CLI inspects an existing project, decides which quality gates apply to it, runs them, reports the outcome, and maps that outcome to an exit status.

## ADDED Requirements

### Requirement: Verification derives the gates that apply
Verification MUST determine which quality gates apply by inspecting the project on disk, without requiring the user to declare a stack or select a gate manually.

#### Scenario: Project with a package manifest
- **WHEN** verification runs against a project whose manifest declares scripts
- **THEN** the gates are derived from those declared scripts

#### Scenario: Project with multiple parts
- **WHEN** verification runs against a project containing more than one recognized part
- **THEN** each part's gates are derived and run against that part's own directory

#### Scenario: Unrecognized project
- **WHEN** verification runs against a directory containing no recognized project
- **THEN** the CLI reports that no gates were found and exits without running any command

### Requirement: Verification runs only gates the project declares
Verification MUST NOT run a command the project does not declare or provide, and MUST report such a gate as not applicable rather than passing or failing it.

#### Scenario: Undeclared script
- **WHEN** a project declares no test script
- **THEN** verification reports the test gate as not applicable and runs no test command

#### Scenario: Missing toolchain
- **WHEN** a gate's tool is not installed on the machine
- **THEN** verification reports that gate as unavailable and continues with the remaining gates

### Requirement: Verification reports each gate and an overall result
Verification MUST report every applicable gate's individual outcome and a single overall result that distinguishes a passed run, a failed gate, and a usage or environment error.

#### Scenario: All applicable gates pass
- **WHEN** every applicable gate succeeds
- **THEN** the CLI reports success and exits with status 0

#### Scenario: A gate fails
- **WHEN** any applicable gate exits non-zero
- **THEN** the CLI names that gate and its command and exits with status 1

#### Scenario: Usage error
- **WHEN** the invocation contains an unrecognized option or a missing option value
- **THEN** the CLI reports the usage error and exits with status 2

#### Scenario: Nothing to verify
- **WHEN** no gate applies to the target directory
- **THEN** the CLI reports that no gates were found and exits with status 3

### Requirement: Verification can gate on dependency vulnerabilities
When the security gate is requested, verification MUST run the dependency vulnerability scanner provided by the project's ecosystem and MUST report a gate for an ecosystem with no bundled scanner as not applicable instead of passing it silently.

#### Scenario: Ecosystem with a bundled scanner
- **WHEN** the security gate is requested for a project whose ecosystem provides a vulnerability scanner
- **THEN** that scanner runs as a gate and its findings determine the gate's outcome

#### Scenario: Ecosystem without a bundled scanner
- **WHEN** the security gate is requested for a project whose ecosystem provides no vulnerability scanner
- **THEN** the CLI reports the security gate as not applicable and states why

### Requirement: Verification can be restricted to a subset of gates
Verification MUST accept a selector that restricts the run to the named gates, so a subset can be re-run without repeating the full sequence.

#### Scenario: Restricting the run
- **WHEN** verification is invoked with a gate selector
- **THEN** only the named gates run and the report contains only those gates

#### Scenario: Selector names an unknown gate
- **WHEN** the selector names a gate that does not apply to the project
- **THEN** the CLI reports a usage error and exits with status 2

### Requirement: CI and users share one verification engine
The repository's own generation harness MUST verify generated projects through the same engine the `verify` command uses, rather than through a separate command table.

#### Scenario: Harness delegates
- **WHEN** the CI harness verifies a generated project
- **THEN** it invokes the same engine the `verify` command invokes
