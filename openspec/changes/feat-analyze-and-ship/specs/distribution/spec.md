# Spec Delta

## Purpose

Defines how the tool is packaged and delivered: the platform targets built for each release, the install paths offered to users who have no build toolchain, and the integrity information published alongside each release.

## ADDED Requirements

### Requirement: Each release publishes a self-contained archive per supported platform
A tagged release MUST publish, for every supported platform, an archive containing the executable, the license, and the source revision it was built from.

#### Scenario: Tagged release publishes archives
- **WHEN** a version tag is pushed
- **THEN** an archive is published for each supported platform

#### Scenario: Archive contents
- **WHEN** an archive is published
- **THEN** it contains the executable, the license file, and the revision it was built from

### Requirement: The documented platform list matches the built target list
The platforms the documentation claims to support MUST match the targets the release process actually builds, in both directions.

#### Scenario: Documented platform is built
- **WHEN** the documentation lists a supported platform
- **THEN** a release archive for that platform is published

#### Scenario: Built platform is documented
- **WHEN** the release process builds a target
- **THEN** that platform is listed in the documentation

### Requirement: Published archives carry verifiable integrity information
Every published archive MUST have a published checksum and a signature that a user can verify without a shared secret.

#### Scenario: Checksum published
- **WHEN** an archive is published
- **THEN** a checksum file for that archive is published alongside it

#### Scenario: Signature verifiable
- **WHEN** a user verifies a downloaded archive against the published signature
- **THEN** the verification succeeds against the documented identity without a shared secret

### Requirement: Release validation is derived from the target list
Release validation MUST derive the number and names of the artifacts it expects from the release's own target list, so adding or removing a platform cannot break the release job.

#### Scenario: Adding a platform
- **WHEN** a platform is added to the target list
- **THEN** the release job expects the additional archive without a second edit

#### Scenario: A missing archive fails the release
- **WHEN** a target builds no archive
- **THEN** the release job fails instead of publishing an incomplete release

### Requirement: Users can install without a build toolchain
The project MUST offer an install path that needs no source build, for each supported shell family, and that path MUST be documented.

#### Scenario: Install on a POSIX shell
- **WHEN** a user follows the documented install steps on a POSIX system
- **THEN** the release binary is fetched and installed without compiling

#### Scenario: Install on Windows
- **WHEN** a user follows the documented install steps on Windows
- **THEN** the release binary is fetched and installed without compiling

### Requirement: Install fails loudly rather than placing an unverified binary
An install that cannot fetch or verify a release asset MUST exit non-zero with an explanatory message and MUST NOT leave a partial or unverified binary on the install path.

#### Scenario: Fetch failure
- **WHEN** the install step cannot fetch the release asset
- **THEN** it exits non-zero, explains the failure, and installs nothing

#### Scenario: Integrity check failure
- **WHEN** a downloaded asset fails its integrity check
- **THEN** the install aborts without placing the binary on the install path
