# Spec Delta

## Purpose

Defines how a generated project is configured for AI coding agents: which agent tools are targeted, where each tool's instruction artifacts are written and in what format, how tool coverage is described in data, and how a machine reads back the result.

## ADDED Requirements

### Requirement: The user selects the AI tools a project targets
Generation MUST accept a selection of AI agent tools, MUST support selecting none, and MUST be able to take that selection without prompting.

#### Scenario: Selecting tools interactively
- **WHEN** generation runs interactively
- **THEN** the user can select one or more AI tools, or none

#### Scenario: Selecting tools without prompts
- **WHEN** generation is invoked with an AI tool selection option
- **THEN** the selected tools are targeted and no prompt is shown

#### Scenario: Selecting no tools
- **WHEN** no AI tool is selected
- **THEN** no AI agent artifact is written and the rest of the generated project is unchanged

### Requirement: Each targeted tool receives artifacts at its documented locations
For every selected tool, generation MUST write that tool's instruction artifacts at the paths and in the formats that tool documents, and MUST NOT write an artifact for a tool that was not selected.

#### Scenario: Cross-tool instruction file
- **WHEN** a cross-tool agent instruction file is selected
- **THEN** it is written at the project root

#### Scenario: Tool with a dedicated directory
- **WHEN** a tool whose artifacts live in a dedicated directory is selected
- **THEN** its artifacts are written inside that directory at the documented path and format

#### Scenario: Unselected tool is untouched
- **WHEN** a tool is not selected
- **THEN** no artifact belonging to that tool is written

### Requirement: Tool coverage is data-driven
The supported AI tools, their artifact paths, and their formats MUST be described in data rather than in code, so supporting a new tool does not require changing generation logic.

#### Scenario: Adding a tool
- **WHEN** a registry entry is added describing a tool's artifacts and formats
- **THEN** the tool becomes selectable and its artifacts are generated without a code change

#### Scenario: Registry entry names an unknown format
- **WHEN** a registry entry names a format the renderer does not implement
- **THEN** generation reports the unknown format instead of silently omitting the artifact

### Requirement: AI artifacts are confined to the project
Generation MUST NOT create or modify any file outside the destination project tree, including any agent configuration in the user's home directory.

#### Scenario: No global agent configuration
- **WHEN** any AI tool is selected
- **THEN** no file under a home-directory agent configuration is created or modified

### Requirement: Generation reports its result as structured data
Generation MUST be able to emit its outcome — the selected tools, the artifacts written, and the result — as machine-readable structured data, without interleaving human-facing progress text.

#### Scenario: Structured result
- **WHEN** generation is invoked requesting machine-readable output
- **THEN** it emits structured data naming the artifacts written and the outcome

#### Scenario: Channels do not interleave
- **WHEN** machine-readable output is requested
- **THEN** human-facing progress text does not appear within the structured result
