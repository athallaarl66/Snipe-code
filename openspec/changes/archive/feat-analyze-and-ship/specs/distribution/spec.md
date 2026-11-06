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

### Requirement: The package manifest carries complete publication metadata
The package manifest MUST declare everything a registry requires to accept a publish, and the license it declares MUST name the same license as the repository's license file.

#### Scenario: Registry-required metadata is present
- **WHEN** the package manifest is inspected
- **THEN** it declares a description, a license, a repository, a readme, keywords, and categories

#### Scenario: Declared license matches the license file
- **WHEN** the license declared in the manifest is compared with the repository license file
- **THEN** both name the same license

#### Scenario: Packaging check passes
- **WHEN** the packaging check runs against the manifest without uploading anything
- **THEN** it completes without a missing-metadata error

### Requirement: A release publishes the crate to the package registry
A tagged release MUST publish the crate version to the ecosystem's package registry, so the tool can be installed through that ecosystem's own installer without cloning the repository.

#### Scenario: Tagged release publishes the crate
- **WHEN** a version tag is pushed
- **THEN** the crate version is published to the registry

#### Scenario: Registry version matches the release tag
- **WHEN** the release completes
- **THEN** the version on the registry matches the release tag

#### Scenario: Nothing is published while the quality gate is red
- **WHEN** the quality gate fails
- **THEN** no crate version is published

#### Scenario: Re-running a partially failed release
- **WHEN** the release is re-run and the registry already holds that version
- **THEN** the publish step reports the existing version and does not fail the run

### Requirement: The tool is installable through the ecosystem package manager
The tool MUST be installable through its own ecosystem's package manager, as an alternative path for users who have that ecosystem's toolchain.

#### Scenario: Install through the package manager
- **WHEN** a user with that toolchain installs the tool through the package manager
- **THEN** the released version is installed without cloning the repository

### Requirement: Repository presentation states what the project is
The repository MUST carry a description and topics naming what the tool is and what it does, and MUST NOT link a homepage that does not exist.

#### Scenario: Description present
- **WHEN** the repository is viewed
- **THEN** it shows a description of the tool

#### Scenario: Topics cover the capabilities
- **WHEN** the repository topics are inspected
- **THEN** they name the ecosystem, the implementation language, and what the tool does

#### Scenario: No placeholder homepage
- **WHEN** no project site exists
- **THEN** the repository homepage is unset rather than pointing at a placeholder
