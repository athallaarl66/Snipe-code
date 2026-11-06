# Spec Delta

## Purpose

Defines where the scaffolder resolves stack, template, and preset content from, including how a local override takes precedence over the built-in defaults shipped inside the binary.

## ADDED Requirements

### Requirement: The binary is self-contained
Building and running the CLI MUST NOT require template content to be present on the filesystem; the built-in templates are part of the binary.

#### Scenario: No template directory present
- **WHEN** the CLI generates a project from a working directory that contains no template directory
- **THEN** generation uses the built-in templates and succeeds

#### Scenario: No network access
- **WHEN** the CLI generates a project with no network connectivity
- **THEN** generation succeeds

### Requirement: A local template directory overrides built-in defaults
When a local template directory is present, the files it provides MUST take precedence over the built-in defaults, leaving every file it does not provide to the default.

#### Scenario: Overriding one file
- **WHEN** a local template directory provides a file that a built-in stack also provides
- **THEN** the generated project contains the local version of that file

#### Scenario: Partial override keeps the remainder
- **WHEN** a local template directory provides only some of a stack's files
- **THEN** the generated project still contains the built-in version of every file the override does not provide

### Requirement: Template resolution is observable
The CLI MUST report whether built-in or local template content supplied a generated project, so a user can tell which source was used.

#### Scenario: Source is reported
- **WHEN** a project is generated
- **THEN** the CLI states whether built-in or local template content was used
