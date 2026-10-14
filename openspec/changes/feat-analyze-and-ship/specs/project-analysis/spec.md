# Spec Delta

## Purpose

Defines what the tool reports about an existing project — its languages and their size, its structure and recognized parts, its entry points, and its declared dependencies — and the form that report takes.

## ADDED Requirements

### Requirement: Analysis is read-only and works without network access
Analysis MUST NOT modify the target project and MUST NOT require network access.

#### Scenario: Project is unchanged
- **WHEN** analysis runs against a project
- **THEN** no file in that project is created, modified, or deleted

#### Scenario: No network access
- **WHEN** analysis runs with no network connectivity
- **THEN** it completes successfully

### Requirement: Analysis reports the languages present and their size
Analysis MUST report each language detected in the project with its own size measure.

#### Scenario: Multiple languages
- **WHEN** analysis runs against a project containing files in more than one language
- **THEN** the report lists each detected language with its own file and line totals

#### Scenario: Excluded content is not counted
- **WHEN** the project contains dependency, build-output, or version-control directories
- **THEN** their contents are excluded from the reported totals

### Requirement: Analysis reports the project's structure and entry points
Analysis MUST describe the project's top-level structure and name the entry points it recognizes.

#### Scenario: Structure reported
- **WHEN** analysis runs against a project
- **THEN** the report names the top-level directories and the parts it recognized

#### Scenario: Entry points reported
- **WHEN** the project contains a recognizable entry point for a part
- **THEN** the report names it

### Requirement: Analysis reports declared dependencies
Analysis MUST report the dependencies the project declares, grouped by the manifest that declares them.

#### Scenario: Dependencies grouped by manifest
- **WHEN** the project contains one or more dependency manifests
- **THEN** the report lists each manifest and the dependencies it declares

#### Scenario: Manifest declaring nothing
- **WHEN** a recognized manifest declares no dependencies
- **THEN** the report names that manifest and states it declares none

### Requirement: Analysis reports its own limits
When analysis cannot recognize content in a project, it MUST state which content was not analyzed rather than presenting an incomplete report as complete.

#### Scenario: Unrecognized content
- **WHEN** the project contains a language or manifest analysis does not handle
- **THEN** the report states which content it did not analyze

### Requirement: Analysis output is available as structured data
Analysis MUST be able to emit its report as structured machine-readable data containing the same findings as the human-readable report.

#### Scenario: Structured report
- **WHEN** analysis is invoked requesting structured output
- **THEN** it emits the report as structured data containing the same findings as the human-readable form
