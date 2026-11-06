## Purpose

Defines trustworthy release artifacts and quality gates for SNIPE-CODE so published binaries correspond to a pinned toolchain, pass repository and generated-project checks, and can be verified by users.

## ADDED Requirements

### Requirement: Rust toolchain is pinned
The repository SHALL declare one tested Rust toolchain version used by local CI and release jobs. The package SHALL declare its minimum supported Rust version, and CI SHALL fail when the active compiler does not satisfy the declared edition and minimum-version policy.

#### Scenario: Pinned CI toolchain
- **WHEN** root CI starts
- **THEN** it installs and reports the repository-pinned Rust version before running checks

#### Scenario: Unsupported minimum version
- **WHEN** minimum-version verification runs on a compiler older than the package declaration
- **THEN** the job fails and identifies the required minimum version

### Requirement: Root quality gate is mandatory
Every protected change SHALL run formatting verification, Clippy with warnings denied, the complete Rust test suite, and the generated-project quality suite defined by this change before merge. A skipped target stack or unavailable required toolchain SHALL fail the release-quality job.

#### Scenario: Clean repository
- **WHEN** code and generated-project checks pass
- **THEN** the root quality job succeeds

#### Scenario: Clippy warning
- **WHEN** Clippy reports a warning under the configured lint gate
- **THEN** CI fails

#### Scenario: Generated stack regression
- **WHEN** one advertised stack fails install, build, or declared test
- **THEN** root CI fails and identifies that stack

### Requirement: Release artifacts cover supported platforms
A release SHALL produce archives for every supported host platform and architecture. Archive contents SHALL include the executable, applicable extension, license, README, and checksums, and SHALL exclude build caches and temporary test data.

#### Scenario: Multi-platform release
- **WHEN** a version release is produced
- **THEN** artifacts exist for each documented supported platform and architecture and are attached to that version

#### Scenario: Archive inspection
- **WHEN** a user inspects a release archive
- **THEN** it contains the documented executable and metadata without source build caches

### Requirement: Release artifacts are checksummed and signed
Every release archive SHALL have a published cryptographic checksum and a verifiable signature or provenance attestation bound to the archive and source revision. Missing signatures, checksums, or provenance SHALL block publication.

#### Scenario: Successful signed release
- **WHEN** release artifacts are published
- **THEN** users can verify archive integrity, signature or provenance, and the referenced source revision using documented commands

#### Scenario: Artifact modified after signing
- **WHEN** a published archive no longer matches its checksum or provenance
- **THEN** verification fails

#### Scenario: Signing unavailable
- **WHEN** release packaging cannot produce required signing material or provenance
- **THEN** the release is not published as a verified release

### Requirement: Release documentation matches artifacts
Installation and verification documentation SHALL identify exact supported platforms, archive names, checksum verification, signature or provenance verification, and source-revision lookup. Documentation SHALL NOT direct users to an unavailable release channel as though a verified release exists.

#### Scenario: User verifies a download
- **WHEN** a user follows release verification documentation
- **THEN** every referenced artifact and verification command corresponds to files published for that release
