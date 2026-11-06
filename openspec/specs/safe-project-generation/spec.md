## Purpose

Defines safe destination and collision behavior for project generation so malformed paths, symlinks, or failed writes cannot cause destructive filesystem changes or silent partial output.

## ADDED Requirements

### Requirement: Project destination is a safe relative name
The generator SHALL accept project names only when they are non-empty, contain no path separator, do not resolve to `.` or `..`, and do not name a filesystem root. The destination SHALL be resolved relative to the current working directory before validation and generation.

#### Scenario: Normal project name
- **WHEN** the user supplies `my-project`
- **THEN** generation targets `./my-project`

#### Scenario: Absolute path
- **WHEN** the user supplies an absolute path such as `C:\work\app`, `/tmp/app`, or `\\server\share\app`
- **THEN** generation is rejected before any filesystem mutation with a message explaining that project name must be a relative name

#### Scenario: Nested path
- **WHEN** the user supplies `apps/website` or `apps\website`
- **THEN** generation is rejected because the value is a path rather than one project directory name

#### Scenario: Traversal component
- **WHEN** the user supplies `.`, `..`, or a value containing a parent-directory component
- **THEN** generation is rejected before any filesystem mutation

### Requirement: Symlink destinations are never recursively deleted
The generator SHALL inspect the destination without following a final symlink. If the destination itself is a symbolic link or junction, generation SHALL reject it rather than deleting, replacing, or writing through it.

#### Scenario: Existing symlink destination
- **WHEN** the requested destination is a symbolic link
- **THEN** generation returns an error and the symlink plus its target remain unchanged

#### Scenario: Symlink inside an existing directory
- **WHEN** an existing ordinary destination contains a symlink and overwrite is confirmed
- **THEN** recursive cleanup does not follow that symlink outside the destination

### Requirement: Existing-directory overwrite requires explicit authorization
An existing ordinary directory SHALL be preserved unless the user has explicitly selected overwrite after being shown the exact canonical destination. Cancel, invalid input, or input failure SHALL leave the existing directory unchanged.

#### Scenario: User cancels collision
- **WHEN** the destination exists and the user selects cancel
- **THEN** no file or directory in the existing destination is changed or removed

#### Scenario: User confirms overwrite
- **WHEN** the destination exists and the user explicitly confirms overwrite after seeing its canonical path
- **THEN** the generator may replace that ordinary directory according to the overwrite contract

### Requirement: Generation is staged before replacing an existing destination
The generator SHALL build required project output in an isolated sibling staging location, validate the staged result, and only then replace the requested destination. A failed generation SHALL remove only generator-owned staging data and SHALL preserve any pre-existing destination.

#### Scenario: Write fails during staging
- **WHEN** any required generated file cannot be written or validated
- **THEN** the requested destination is not created or replaced and no partially generated project remains visible at that destination

#### Scenario: Successful new destination
- **WHEN** all required output is written and validated for a destination that does not exist
- **THEN** the completed output is published atomically at the requested destination

#### Scenario: Successful overwrite
- **WHEN** overwrite is authorized and staged output passes validation
- **THEN** the validated output replaces the existing destination and staging data is removed

### Requirement: Dry run performs no filesystem mutation
Dry-run mode SHALL validate and display the planned output without creating, deleting, replacing, or writing any filesystem entry.

#### Scenario: Dry run for a new project
- **WHEN** the user previews a project that does not exist
- **THEN** no destination or staging directory is created

#### Scenario: Dry run for an existing project
- **WHEN** the user previews overwrite of an existing destination
- **THEN** the existing destination is not changed and the preview clearly marks the planned replacement
