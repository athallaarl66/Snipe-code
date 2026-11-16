## Purpose

Defines evidence-based product documentation so advertised capabilities, security posture, supported platforms, and installation instructions match executable behavior and published artifacts.

## ADDED Requirements

### Requirement: Quality labels require defined evidence
User-facing documentation SHALL NOT describe SNIPE-CODE as “enterprise-grade,” “production-ready,” “security-baked,” or equivalent unless each label is linked to a documented acceptance criterion and passing release-quality check. Until those checks exist and pass, documentation SHALL describe the tool as a scaffold generator with heuristic audit support.

#### Scenario: Unsupported enterprise-grade claim
- **WHEN** no release-quality evidence proves the enterprise-grade acceptance criteria
- **THEN** README and product documentation do not make that claim

#### Scenario: Evidence-backed claim
- **WHEN** a quality label has documented criteria and every required check passes in the release job
- **THEN** documentation may state the label and link users to that evidence

### Requirement: Feature documentation matches generated output
Every advertised frontend, backend, database, Docker, CI, Git, and AI-workspace capability SHALL describe the exact files or behavior generated for supported selections. Features that are optional, best-effort, unsupported, or deferred SHALL be labeled accordingly.

#### Scenario: AI workspace feature
- **WHEN** documentation advertises AI workspace support
- **THEN** it names the files actually generated and does not claim agent compatibility beyond verified discovery behavior

#### Scenario: Git automation
- **WHEN** documentation advertises automatic Git initialization and commit
- **THEN** it discloses behavior when Git is unavailable or commit identity is not configured

#### Scenario: Docker automation
- **WHEN** documentation advertises Docker Compose generation
- **THEN** it identifies which selected stacks receive backend containers and which receive database-only services

### Requirement: Security documentation states scope and limits
Security-related documentation SHALL distinguish configuration scaffolding from enforced controls and heuristic audit findings from proven vulnerability absence. It SHALL disclose supported audit contexts, excluded paths, false-positive risk, and any unaudited scope.

#### Scenario: Generated security configuration
- **WHEN** documentation lists `.env.example` or CORS/Helmet settings
- **THEN** it states whether these files configure controls, install middleware, or are only placeholders

#### Scenario: Empty audit result
- **WHEN** documentation describes an audit with no findings
- **THEN** it explains that absence of findings is limited to evaluated heuristics and does not prove security

### Requirement: Installation and release instructions are executable
Documented build, install, generated-project setup, and release-verification commands SHALL exist and target supported toolchain versions. Commands and paths mentioned in documentation SHALL be checked by automated documentation or smoke verification.

#### Scenario: Generated setup command
- **WHEN** a generated README lists an install or build command
- **THEN** the referenced manifest or file exists and the command succeeds in the corresponding stack verification job

#### Scenario: Release verification command
- **WHEN** documentation provides checksum or signature verification commands
- **THEN** those commands work against the documented release artifact layout

### Requirement: Test claims are reproducible
Documentation SHALL describe quality checks using commands users or contributors can run and SHALL NOT use unsupported test counts, pass claims, or completion records as enduring evidence. Durable CI results and generated-project verification output are the source of truth for current pass status.

#### Scenario: Test count changes
- **WHEN** tests are added or removed
- **THEN** documentation does not require manual count updates to remain accurate

#### Scenario: Verification claim
- **WHEN** documentation states that a supported stack builds or tests successfully
- **THEN** that behavior is covered by a named automated verification job
