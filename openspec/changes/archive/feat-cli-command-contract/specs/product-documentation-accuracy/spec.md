## ADDED Requirements

### Requirement: Command documentation states its run contract

Documentation for every user-facing command SHALL state the directory the command must be run from, the paths it reads and writes, and its exit codes. Behavior that changes the filesystem SHALL be identified as such rather than implied by a warning about network use.

#### Scenario: Scaffolding command
- **WHEN** the scaffolding command is documented
- **THEN** the documented destination is `<current directory>/<project name>` and the text states that a path argument does not relocate the output

#### Scenario: Report-producing command
- **WHEN** the audit command is documented
- **THEN** the report file names and the directory they are written into are named, together with the format used when no terminal is present

#### Scenario: Verification command
- **WHEN** the verification command is documented
- **THEN** the files its gates write inside the target project are named, and the option that skips the install gate is documented with its exit-code consequences

#### Scenario: Non-interactive defaults
- **WHEN** documentation describes behavior without a terminal
- **THEN** the defaults are listed by value instead of being summarized as "defaults"

## MODIFIED Requirements

### Requirement: Installation and release instructions are executable

Documented build, install, generated-project setup, and release-verification commands SHALL exist and target supported toolchain versions. Commands and paths mentioned in documentation SHALL be checked by automated documentation or smoke verification.

#### Scenario: Generated setup command
- **WHEN** a generated README lists an install or build command
- **THEN** the referenced manifest or file exists and the command succeeds in the corresponding stack verification job

#### Scenario: Release verification command
- **WHEN** documentation provides checksum or signature verification commands
- **THEN** those commands work against the documented release artifact layout

#### Scenario: Cargo install from a checkout
- **WHEN** documentation gives a cargo install command for a local checkout
- **THEN** the documented command succeeds as written, including any flag required because a transitive dependency of the crate is yanked on crates.io
