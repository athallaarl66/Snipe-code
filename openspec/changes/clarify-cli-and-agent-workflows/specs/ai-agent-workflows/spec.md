## Purpose

Defines honest project-local workflows for AI coding agents while keeping deterministic CLI execution separate from agent reasoning and code changes.

## ADDED Requirements

### Requirement: AI integration explains what is generated before selection
The project wizard SHALL explain that selecting an AI coding agent writes project-local instruction, rule, skill, or workflow files and does not run a model, authenticate a provider, store credentials, or require an API key.

#### Scenario: AI integration prompt
- **WHEN** the wizard reaches optional AI integration
- **THEN** the user sees the boundary explanation before choosing any agent

#### Scenario: No agent selected
- **WHEN** the user selects no AI agent
- **THEN** no selected-agent artifact is written and project generation continues normally

### Requirement: Agent compatibility is reported per artifact type
The registry, wizard summary, generated result, and documentation SHALL distinguish rules, skills, and invocable commands or workflows for each supported agent instead of implying identical capabilities.

#### Scenario: Agent with an invocable workflow
- **WHEN** an agent receives a slash command or workflow artifact
- **THEN** generated output names that invocable artifact and how the agent exposes it

#### Scenario: Agent with instructions only
- **WHEN** an agent receives only rules, a root instruction file, or an auto-loaded skill
- **THEN** generated output does not claim that the agent supports a generated slash command

### Requirement: Invocable workflows use explicit task names
Agents that support generated invocable workflows SHALL receive separate audit, verification, and test-authoring workflows rather than one generic `/snipe-code` workflow whose behavior is limited to auditing.

#### Scenario: Audit workflow
- **WHEN** a user invokes the generated SNIPE-CODE audit workflow
- **THEN** the agent runs the deterministic audit, reviews findings against source, fixes only confirmed risks when authorized, runs project tests, and repeats the audit

#### Scenario: Verification workflow
- **WHEN** a user invokes the generated SNIPE-CODE verification workflow
- **THEN** the agent runs the deterministic verification command, diagnoses failures from source and command output, applies only requested fixes, and verifies the result again

#### Scenario: Test-authoring workflow
- **WHEN** a user invokes the generated SNIPE-CODE test-authoring workflow with a requested behavior
- **THEN** the agent reads the relevant source and requirements, asks when expected behavior is ambiguous, writes or updates tests, and runs `snipe-code verify` with the test gate

### Requirement: Test authoring and test execution have separate ownership
Generated guidance SHALL state that AI agents may author or repair tests through their coding workflow, while the executable discovers and runs declared tests and produces deterministic results and reports.

#### Scenario: Executable verification
- **WHEN** `snipe-code verify` runs without an agent
- **THEN** it executes existing declared tests and does not claim to generate test cases

#### Scenario: Agent authors a test
- **WHEN** an agent workflow creates a test
- **THEN** the agent runs that test through the project toolchain or SNIPE-CODE verification before claiming it works

### Requirement: Workflow generation remains project-local and provider-neutral
AI workflow artifacts SHALL remain inside the generated project and SHALL invoke the installed `snipe-code` executable without embedding provider credentials or model SDKs.

#### Scenario: Workflow artifacts generated
- **WHEN** any supported agent workflow is selected
- **THEN** no home-directory configuration, credential, API key, or embedded provider dependency is created
