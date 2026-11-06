## ADDED Requirements

### Requirement: Quick start matches currently usable distribution
The primary README SHALL lead with a concise product description, a working quick start verified on a clean checkout, and current availability of release assets, crate, and image. Unavailable channels SHALL be labeled unavailable instead of shown as working installation paths; stale status SHALL be checked before publication.

#### Scenario: Release has no assets
- **WHEN** the latest published release has no downloadable archives
- **THEN** the README directs users to a tested source-build path and labels installer download as unavailable

#### Scenario: Registry entry absent
- **WHEN** the crate is not published
- **THEN** the README does not present `cargo install snipe-code --locked` as an immediately working command

### Requirement: Readme helps users choose commands before showing flags
The README SHALL distinguish where to run `new`, `analyze`, `audit`, and `verify`, including their write, network, and commit side effects. Its quick start SHALL use an isolated destination rather than suggesting a destructive diagnostic on the repository root.

#### Scenario: User wants an offline read-only report
- **WHEN** a user selects a command from the guide
- **THEN** `analyze` is identified as read-only, while `audit` is identified as writing report files and `verify` as able to run networked project tools

#### Scenario: User creates a project
- **WHEN** a user follows the scaffold example
- **THEN** it explains the current-directory destination and best-effort Git initialization and initial commit

### Requirement: Documentation separates demonstrated behavior from plans
The primary README and English feature documents SHALL identify implemented behavior, known limitations, and deferred proposals using current CLI flags, preset IDs, generated paths, and real test evidence. Visual elements and recorded demos SHALL reflect actual output and remain readable in plain text and narrow displays.

#### Scenario: Outdated feature document
- **WHEN** a feature document mentions an unimplemented flag or template
- **THEN** it labels that behavior planned or removes the misleading example instead of presenting it as current

#### Scenario: README identity and demo
- **WHEN** the README presents an ASCII banner or terminal example
- **THEN** it remains legible at narrow width and any displayed program output comes from an actual run

### Requirement: AI integration instructions distinguish generated files from services
Documentation SHALL explain that agent artifacts are optional files for external AI tools, that the executable does not embed an AI service, and which artifact paths and slash commands apply to a selected tool. It SHALL state prerequisites and caution that heuristic HIGH findings require review.

#### Scenario: AI opt-in documentation
- **WHEN** a user reads the AI section before selection
- **THEN** it explains what files are always generated, which files require selection, and that `/snipe-code` exists only for tools with a generated command artifact
