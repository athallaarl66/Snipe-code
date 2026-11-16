## ADDED Requirements

### Requirement: Published channel status names its evidence
Release-facing documentation SHALL distinguish an artifact attached to a release, an accessible package version, a completed publishing job, and a successful user-facing installation test. A channel SHALL be described as tested only where its specific published version and platform were actually exercised.

#### Scenario: Archive published without install smoke
- **WHEN** a signed archive is available but the installer has not been executed on that target
- **THEN** documentation describes the artifact as published and the install test as pending

#### Scenario: Published crate with successful CI
- **WHEN** crates.io lists the version and CI passes but no clean registry install was run
- **THEN** documentation does not turn those two checks into a claim that registry installation was tested
