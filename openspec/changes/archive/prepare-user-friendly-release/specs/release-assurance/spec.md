## ADDED Requirements

### Requirement: Each advertised install channel has publication evidence
A release SHALL advertise an installation channel as available only after its corresponding published artifact can be fetched and exercised on a supported target. Local build success or a green main-branch CI alone SHALL NOT count as proof that release archives, crate, or container image exist.

#### Scenario: GitHub release installer advertised
- **WHEN** the README gives a working shell installer command
- **THEN** a real release archive and checksum exist for that target, and the installer has installed the published executable in a fresh temporary destination

#### Scenario: Crate advertised
- **WHEN** the README gives a working registry install command
- **THEN** the registry contains the specified crate version and a clean `cargo install ... --locked` succeeds

#### Scenario: Container image advertised
- **WHEN** the README gives a working image invocation
- **THEN** the image tag exists and the published image runs the documented command

### Requirement: Failed publication does not masquerade as a complete release
Release readiness SHALL be based on published archive count, checksum and signature verification, installer smoke tests, registry state, and image state separately. A failed job or empty public release SHALL be disclosed until a corrected release is independently verified.

#### Scenario: Empty release
- **WHEN** a tag produces a public release with no assets
- **THEN** documentation does not mark binary installation ready, even if the main branch quality workflow passed
