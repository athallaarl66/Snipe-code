# Spec Delta

## ADDED Requirements

### Requirement: A generated project can carry artifacts for multiple agent tools
A generated project MUST be able to carry instruction artifacts for more than one AI agent tool at the same time, each placed according to that tool's own convention.

#### Scenario: Several tools selected
- **WHEN** more than one AI tool is selected
- **THEN** the project contains an artifact for each selected tool, at that tool's own location

#### Scenario: Exactly one tool selected
- **WHEN** exactly one AI tool is selected
- **THEN** the project contains that tool's artifacts and no other tool's

### Requirement: Shared guidance is consistent across the artifacts written
The guidance common to every selected tool MUST NOT contradict itself between the artifacts generated for different tools in the same project.

#### Scenario: Shared rules agree
- **WHEN** several tools whose artifacts share common guidance are selected
- **THEN** the shared guidance is consistent across the artifacts written for them

### Requirement: The workspace directory keeps the shared rules content
A generated project MUST continue to provide the shared rules content in the workspace directory for tools that read no dedicated convention.

#### Scenario: Workspace rules still present
- **WHEN** a project is generated with AI artifacts selected
- **THEN** the workspace directory still contains the shared rules files
