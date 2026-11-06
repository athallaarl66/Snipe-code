## Purpose

Audit discovers project contexts recursively and identifies ecosystem/framework from content. Reports distinguish evaluated, unknown, partial, and excluded scope.

## ADDED Requirements

### Requirement: Discover project contexts recursively
The system SHALL walk the audit target recursively, identify recognized manifest locations, and create a project context for each recognized context, including contexts below `frontend/` and `backend/`. Traversal SHALL be bounded and SHALL record unreadable or truncated scope.

#### Scenario: Generated frontend and backend contexts
- **WHEN** audit runs on a generated project containing `frontend/package.json` and `backend/composer.json`
- **THEN** the report lists separate frontend and backend contexts and evaluates rules within each context

#### Scenario: Nested monorepo contexts
- **WHEN** audit runs on a directory containing nested manifests under multiple subdirectories
- **THEN** each recognized context is reported with its root path and owned scope

#### Scenario: Traversal limit is disclosed
- **WHEN** recursive discovery reaches its configured depth or traversal limit
- **THEN** the report marks scope as partial and identifies the unvisited scope instead of claiming a complete scan

### Requirement: Exclude paths without following symlinks
The system SHALL skip hidden directories, VCS/dependency/build/report directories, and symlink paths; it SHALL NOT follow directory symlinks or read symlink files, and SHALL report each excluded path with its reason.

#### Scenario: Dependency path excluded
- **WHEN** a context contains `node_modules`, `vendor`, `target`, or `obj`
- **THEN** files below that path are not evaluated and the path is listed as excluded

#### Scenario: Directory symlink excluded
- **WHEN** discovery encounters a symlink to another directory
- **THEN** it does not recurse into the symlink and records the symlink path and reason `symlink`

#### Scenario: Symlink loop is safe
- **WHEN** a directory symlink points to an ancestor of itself
- **THEN** discovery terminates without traversing the symlink repeatedly

### Requirement: Distinguish ecosystem from framework
The system SHALL identify ecosystem from recognized manifest content and SHALL identify framework only from content markers. A known ecosystem with no framework marker SHALL report framework `unknown`; a misleading or invalid manifest SHALL report partial/unknown identity with its reason.

#### Scenario: JavaScript framework markers
- **WHEN** `package.json` contains `next`, `react`, `vue`, or `nuxt` dependencies or scripts
- **THEN** the context reports the corresponding JavaScript/TypeScript framework marker

#### Scenario: PHP is not automatically Laravel
- **WHEN** `composer.json` exists without a `laravel/framework` marker
- **THEN** the context reports PHP and framework `unknown`, not Laravel

#### Scenario: Java is not automatically Spring
- **WHEN** `pom.xml` or `build.gradle` exists without a Spring marker
- **THEN** the context reports Java and framework `unknown`, not Spring

#### Scenario: Rust identity
- **WHEN** a context contains `Cargo.toml`
- **THEN** the context reports the Rust ecosystem and is eligible for Rust rules

### Requirement: Report evaluated and unknown scope
The system SHALL surface detected contexts, ecosystem/framework identity, evaluated rule groups, evaluated file count, unknown scope, partial scope, and excluded paths in every audit report.

#### Scenario: Unknown project context
- **WHEN** no recognized manifest is found
- **THEN** the report identifies the scope as unknown, runs only common rules if applicable, and does not claim language-specific coverage

#### Scenario: No findings in evaluated rules
- **WHEN** evaluated rules produce no findings
- **THEN** the report says `No findings detected by evaluated rules` and still lists evaluated and unknown scope
