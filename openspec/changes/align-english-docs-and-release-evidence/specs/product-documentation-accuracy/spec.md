## ADDED Requirements

### Requirement: Published Markdown is accurate English
The repository SHALL provide README and published product, technical, production, test-plan, and generated-agent Markdown guidance in English. Documentation SHALL retain CLI literals and source identifiers verbatim while separating implemented behavior from historical examples and planned features.

#### Scenario: Unsupported historical example
- **WHEN** a legacy scenario shows an absent preset, unsupported option, or nonexistent source type as executable today
- **THEN** the page replaces it with a working case or clearly labels it historical and not runnable

#### Scenario: CLI message is not in English
- **WHEN** a command emits a non-English message that must be quoted for troubleshooting
- **THEN** documentation may preserve that literal and give an English explanation without inventing replacement output

### Requirement: Documentation index lists real artifacts
The documentation index SHALL link only existing documentation and SHALL label draft scenario specifications differently from executed test results. It SHALL NOT claim a report directory or documentation generator command exists if it is absent.

#### Scenario: Reader looks for UAT results
- **WHEN** the index links UAT cases or a release report
- **THEN** it identifies whether the linked item is a plan, executed result, or unfilled template

### Requirement: Quick start reflects published release state
The README SHALL identify the latest verified published version and distinguish available archives, crates.io package, image publication, and platform-specific installation smoke results. The first runnable path SHALL have prerequisites and an executable command matching the tested platform.

#### Scenario: New release is published
- **WHEN** a newer version is publicly available but the README still describes an older release as the latest
- **THEN** the README is considered stale until the version and distribution claims are reconciled with the public artifacts
