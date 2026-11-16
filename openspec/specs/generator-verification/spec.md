## Purpose

Defines executable verification of generated projects so file-presence tests cannot be the only evidence that advertised stacks and integrations work.

## ADDED Requirements

### Requirement: Generated-project verification uses isolated destinations
Each generated-project verification SHALL run in a unique temporary or CI workspace outside repository fixture paths and SHALL clean up only locations created by that verification run.

#### Scenario: Parallel verification
- **WHEN** multiple generated-project verifications run concurrently
- **THEN** each uses a distinct destination and none deletes or overwrites another run's workspace

#### Scenario: Verification cleanup
- **WHEN** a verification completes successfully or fails
- **THEN** only its own temporary workspace is removed and repository files remain unchanged

### Requirement: Every advertised stack has a native smoke verification
Each advertised frontend and backend stack SHALL have at least one automated verification that generates the stack and runs its declared install and build commands with the documented toolchain. Verification SHALL fail when a required toolchain is unavailable in the release-quality job rather than silently skipping the stack.

#### Scenario: Frontend stack matrix
- **WHEN** release-quality verification runs
- **THEN** Next.js, React Vite, Vue, and Nuxt are each generated, installed, and built in isolated workspaces

#### Scenario: Backend stack matrix
- **WHEN** release-quality verification runs
- **THEN** NestJS, Go Gin, Spring Boot, .NET 8, and Laravel are each generated and built or tested in isolated workspaces

#### Scenario: Missing verification toolchain
- **WHEN** a required target toolchain is not available in the release-quality job
- **THEN** verification fails with the missing toolchain and affected stack named

### Requirement: Generated CI definitions are structurally and semantically checked
Generated GitHub Actions YAML SHALL be parsed as YAML and checked for valid workflow structure. Before release, verification SHALL confirm that each referenced local path exists and every package script invoked by a Node workflow is declared by that package manifest.

#### Scenario: Valid generated workflow
- **WHEN** CI is generated for a supported stack
- **THEN** YAML parsing succeeds and referenced working directories exist

#### Scenario: Missing npm script
- **WHEN** a generated workflow invokes a Node script absent from `package.json`
- **THEN** verification fails and identifies both the missing script and generated workflow

### Requirement: Docker output is validated when generated
When Docker Compose is generated, automated verification SHALL validate the Compose model and every referenced image or local build context. If the verification environment does not support Docker, the release-quality job SHALL be considered incomplete rather than reporting the Docker contract as passed.

#### Scenario: Valid compose output
- **WHEN** a database-only or supported backend Compose file is generated
- **THEN** Compose validation succeeds and all referenced local paths exist

#### Scenario: Missing build context
- **WHEN** Compose references a backend build context without a Dockerfile
- **THEN** verification fails with the missing Dockerfile named

### Requirement: Tests never target fixed destructive repository paths
Automated tests that invoke project generation SHALL allocate unique temporary destinations. Tests SHALL NOT recursively remove fixed directories under the repository or developer working directory.

#### Scenario: Repeated test execution
- **WHEN** the same test suite runs more than once
- **THEN** prior unrelated files and directories remain untouched

#### Scenario: Test interruption
- **WHEN** a generation test stops after creating its temporary destination
- **THEN** no fixed repository fixture path is deleted by cleanup or panic handling
