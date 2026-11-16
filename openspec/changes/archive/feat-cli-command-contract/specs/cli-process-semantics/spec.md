## ADDED Requirements

### Requirement: Verification can skip the install gate

The `verify` command SHALL accept `--no-install`. With the flag set, the install gate SHALL run no command and SHALL be reported as `not-applicable` with a reason that names the flag, while every other applicable gate still runs. Combining `--no-install` with `--only install` SHALL be rejected as a usage error before any gate runs.

#### Scenario: Install gate skipped, the rest still runs
- **WHEN** a recognized project declares an install gate and the user runs `verify . --no-install`
- **THEN** the install gate is reported `not-applicable` with the reason `skipped by --no-install` and the remaining gates run in the fixed order

#### Scenario: Skipping leaves nothing to execute
- **WHEN** the install gate is the only gate that applies and `--no-install` is set
- **THEN** no command runs and the process exits with code `3`, the same outcome as a project with no runnable gate

#### Scenario: Contradictory selectors
- **WHEN** `--no-install` is combined with `--only install`
- **THEN** the process writes a usage error to standard error and exits with code `2` without running a single gate

#### Scenario: Default run is unchanged
- **WHEN** `verify` runs without `--no-install`
- **THEN** the install gate runs exactly as it did before the option existed, including its exit code contribution
