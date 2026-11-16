## Purpose

Defines a safe English entry experience that lets interactive users choose a goal before any command runs or changes project files.

## ADDED Requirements

### Requirement: Interactive no-argument invocation presents user goals
When SNIPE-CODE is started without a command in an interactive terminal, the CLI SHALL present an English intent menu before running any product operation.

#### Scenario: Launcher opens
- **WHEN** a user starts `snipe-code` or opens `snipe-code.exe` without arguments in an interactive terminal
- **THEN** the CLI offers `Create a new project`, `Understand an existing project`, `Run tests and quality checks`, `Audit project security`, `View help`, and `Exit`

#### Scenario: No action runs before selection
- **WHEN** the launcher is waiting for a choice
- **THEN** no project is generated, scanned, audited, verified, or modified

### Requirement: Launcher choices route to existing product operations
Each launcher choice SHALL route to the corresponding existing command flow without removing or changing the availability of direct commands.

#### Scenario: Create route
- **WHEN** the user selects `Create a new project`
- **THEN** the CLI enters the same generation flow available through `snipe-code new`

#### Scenario: Existing-project routes
- **WHEN** the user selects understand, quality checks, or security audit
- **THEN** the CLI enters the corresponding `analyze`, `verify`, or `audit` flow and identifies the target project before execution

#### Scenario: Direct command remains available
- **WHEN** a user invokes `snipe-code verify PATH` directly
- **THEN** verification starts without showing the launcher

### Requirement: Interactive product UI uses English consistently
The launcher and every interactive flow reachable from it SHALL use English for prompts, option labels, descriptions, navigation guidance, summaries, status messages, errors, and cancellation text.

#### Scenario: Project creation prompts
- **WHEN** the user enters project creation from the launcher
- **THEN** template, stack, project name, AI integration, collision, confirmation, and completion text is in English

#### Scenario: Keyboard guidance
- **WHEN** the CLI presents an interactive selection
- **THEN** it explains the applicable arrow, space, Enter, back, cancel, or exit controls in English

### Requirement: Non-interactive no-argument invocation is safe
When SNIPE-CODE is started without a command and no interactive terminal is available, the CLI SHALL print concise usage guidance and exit without filesystem or network activity.

#### Scenario: Script invokes the executable without arguments
- **WHEN** stdin or stdout is non-interactive and no command was supplied
- **THEN** the CLI does not create the default `my-project`, prints how to select a command, and exits with usage status `2`

### Requirement: Launcher exposes help and safe exit
The launcher SHALL let the user read concise command guidance or leave without starting an operation.

#### Scenario: View help
- **WHEN** the user selects `View help`
- **THEN** the CLI displays concise command guidance and allows return to the launcher or exit

#### Scenario: Exit launcher
- **WHEN** the user selects `Exit` or cancels before an operation begins
- **THEN** the process exits successfully without changing files
