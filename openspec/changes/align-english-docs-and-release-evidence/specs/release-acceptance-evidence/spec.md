## Purpose

Defines how a release records reproducible user acceptance scenarios and test evidence without presenting draft cases or CI checks as human approval.

## ADDED Requirements

### Requirement: UAT cases cover supported user journeys
A release SHALL have English UAT cases for installing the CLI, generating and replacing projects safely, dry run, analysis, audit exports, verification side effects, and optional AI artifacts. Each case SHALL state prerequisites, action, expected behavior, and an evidence field without presuming a pass.

#### Scenario: Generated project acceptance
- **WHEN** a tester selects a supported stack and preset
- **THEN** the case specifies expected project paths, command exit status, and how to record observed files and results

#### Scenario: Overwrite safety acceptance
- **WHEN** a tester attempts to replace an existing destination
- **THEN** the case distinguishes cancel, successful overwrite, and failed publication with recovery evidence

### Requirement: Versioned test report distinguishes evidence levels
A versioned test report SHALL identify the tested build or release, platform, case identifier, executed command or action, expected and observed results, pass/fail/blocked/not-run status, evidence location, and outstanding defects. It SHALL distinguish automated CI results from installation smoke tests and human UAT approval.

#### Scenario: CI passes but UAT was not run
- **WHEN** CI reports success but no human executed an acceptance case
- **THEN** the report lists CI as passed and UAT as not run, without a sign-off

#### Scenario: Installer was not tested on a platform
- **WHEN** a release archive exists but an installer was not exercised there
- **THEN** the report leaves that installer case not run or blocked and does not claim the platform smoke test passed

### Requirement: Release decision remains evidence based
A release report SHALL expose unresolved failures and state whether a go/no-go decision was made, by whom, and on which evidence. A blank signature or an unexecuted template SHALL NOT count as acceptance.

#### Scenario: No approval recorded
- **WHEN** test results exist but the authorized reviewer has not approved the release
- **THEN** the report states decision pending rather than asserting release-ready
