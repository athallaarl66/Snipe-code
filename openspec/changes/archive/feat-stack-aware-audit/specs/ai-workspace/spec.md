## ADDED Requirements

### Requirement: Generate AGENTS.md at project root
The system SHALL generate an `AGENTS.md` file at the project root, not in a subfolder, when scaffolding a project, so agents and IDEs that auto-read root context files can pick it up.

#### Scenario: Project generation creates root AGENTS.md
- **WHEN** a project is generated with AI workspace enabled
- **THEN** `AGENTS.md` exists at the project root
- **THEN** the file contains discovered ecosystem/framework context, applicable security rules, code-style conventions, and unknown/partial scope when applicable

#### Scenario: No stray AI workspace folder
- **WHEN** a project is generated
- **THEN** no `.ai-workspace/` folder with rules files is created

### Requirement: No fictional slash commands
The system SHALL NOT include slash commands in generated AI files unless they are actually implemented by the CLI.

#### Scenario: Generated content has no fake commands
- **WHEN** `AGENTS.md` is generated
- **THEN** its content contains no `/command` entries such as `/pentest`, `/fix-smells`, `/gen-docs`, or `/snipe-code`

### Requirement: AI guidance does not overclaim security
Generated `AGENTS.md` SHALL distinguish heuristic audit guidance from proof that a project is secure.

#### Scenario: Unknown framework context
- **WHEN** generated context has an unknown or partial framework identity
- **THEN** `AGENTS.md` states that scope limitation and does not claim complete security coverage
