## ADDED Requirements

### Requirement: Interactive export format selection
When the `audit` command is run without `--export`, the system SHALL prompt the user with an interactive multi-select of export formats (Markdown, PDF, DOCX) using arrow-key navigation.

#### Scenario: No --export flag given
- **WHEN** user runs `snipe-code audit .` without `--export`
- **THEN** an interactive multi-select prompt shows Markdown, PDF, DOCX options
- **THEN** reports are exported in the selected format(s)

#### Scenario: Default selection
- **WHEN** the format prompt is shown
- **THEN** Markdown is pre-selected by default

### Requirement: Non-interactive export flag
The system SHALL keep the `--export <md,pdf,docx>` flag working for non-interactive and scripted use, skipping the prompt.

#### Scenario: --export pdf,docx
- **WHEN** user runs `snipe-code audit . --export pdf,docx`
- **THEN** no prompt is shown and PDF + DOCX reports are exported directly

#### Scenario: At least one format required
- **WHEN** user deselects all formats in the prompt or passes an empty `--export`
- **THEN** CLI prints a warning that at least one format is required and re-prompts or exits with a clear message
