# Spec Delta

## Purpose

Defines the AI-agent instruction artifacts written into a generated project so that agent tools can orient themselves in the project without extra setup.

## ADDED Requirements

### Requirement: Generated projects include the advertised AI agent instruction files
A generated project MUST contain the AI agent instruction files the product documents, written at the project root.

#### Scenario: Cursor rules file
- **WHEN** a project is generated
- **THEN** a `.cursorrules` file exists at the project root

#### Scenario: Agent instructions file
- **WHEN** a project is generated
- **THEN** an `AGENTS.md` file exists at the project root

### Requirement: Generation never writes outside the project root
Scaffolding MUST confine every write, including AI agent artifacts, to the destination project tree.

#### Scenario: No global agent configuration is touched
- **WHEN** a project is generated
- **THEN** no file under the user's home-directory agent configuration is created or modified
