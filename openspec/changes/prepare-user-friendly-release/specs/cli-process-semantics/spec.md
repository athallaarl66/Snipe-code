## ADDED Requirements

### Requirement: New supports explicit stack and preset selection
The `new` command SHALL accept validated template, frontend, backend, and database IDs as options in addition to existing name and AI-tool options. In a terminal it SHALL prompt only for selections that were not supplied; without a terminal it SHALL use documented defaults only for missing selections. Invalid or unsupported combinations SHALL fail before writing any project file.

#### Scenario: Explicit non-interactive selection
- **WHEN** a script supplies valid template, frontend, backend, database, and project name options
- **THEN** the generated project reflects those selections without requiring a terminal

#### Scenario: Partial interactive selection
- **WHEN** the user supplies only a valid backend ID in a terminal
- **THEN** the backend prompt is skipped and the other selections can still be made interactively

#### Scenario: Invalid selection
- **WHEN** a flag names an unknown stack or requests no frontend and no backend
- **THEN** `new` exits with usage error 2 before creating, staging, or replacing a project

### Requirement: New rejects unrecognized or malformed options
The `new` command SHALL reject unknown options, missing option values, and extra positional arguments with usage exit status 2 before invoking generation; the `--dry-run` alias SHALL follow the same validation.

#### Scenario: Typo in option
- **WHEN** the user types an unrecognized option after `new`
- **THEN** the CLI names the option, exits 2, and does not generate the default project

#### Scenario: Flag value missing
- **WHEN** a value-taking option is last or immediately followed by another option
- **THEN** the CLI explains which value is missing and exits 2 without writing

### Requirement: Audit JSON output remains parseable with default export
The `audit --json` command SHALL write only one valid JSON document to standard output whether or not an export format was supplied. Human-facing default-export notices and progress SHALL go to standard error; selected report files SHALL still be written to the target directory.

#### Scenario: Non-interactive default export
- **WHEN** `audit PATH --json` runs without a terminal and without `--export`
- **THEN** stdout parses as one JSON result, the Markdown report exists in PATH, and the default-export notice appears on stderr

#### Scenario: Explicit export
- **WHEN** `audit PATH --export md --json` runs
- **THEN** stdout remains JSON and the Markdown report is written
