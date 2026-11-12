## Purpose

Define bounded regression protection for template override paths and generated database and CI configuration without changing the scaffolding product surface.

## ADDED Requirements

### Requirement: Template override paths stay inside their boundary
Generation MUST validate override folders, file paths, and template sources before staging I/O. Traversal, absolute, drive, UNC, and symlink/reparse escapes MUST be rejected without reading or writing outside the approved boundary or replacing the destination.

#### Scenario: Malicious preset path
- **WHEN** an override supplies an escaping folder, file path, or template source
- **THEN** generation fails before outside-boundary I/O and existing destination and canary files remain unchanged

### Requirement: Generated database configuration is consistent
Generated Compose services and backend connection configuration MUST use matching database identity and credentials and omit obsolete Compose version declarations. Runtime readiness claims MUST distinguish actual disposable connection and healthcheck evidence from config parsing.

#### Scenario: Backend connects to generated database
- **WHEN** an approved disposable Compose acceptance run starts the generated services
- **THEN** backend configuration resolves the same database and credentials as the healthy database service; absent runtime evidence remains NOT RUN

### Requirement: Generated Go CI handles missing lock data
Generated Go CI MUST avoid cache setup that requires an absent go.sum and MUST retain dependency resolution and verification steps.

#### Scenario: Fresh generated Go project
- **WHEN** CI starts from a scaffold without go.sum
- **THEN** setup does not require that file before dependency resolution; actual CI outcome is reported separately when available
