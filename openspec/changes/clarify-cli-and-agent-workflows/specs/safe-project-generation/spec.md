## ADDED Requirements

### Requirement: Interactive generation discloses context before writing
The interactive generation flow SHALL identify the current step, explain navigation, show the resolved destination, and present the selected preset, frontend, backend, database, and AI integrations before any project file or Git state is created or changed.

#### Scenario: Step context
- **WHEN** a user moves through interactive generation
- **THEN** each prompt identifies its step and gives applicable navigation guidance

#### Scenario: Configuration summary
- **WHEN** all generation inputs are valid
- **THEN** the CLI displays the exact resolved destination and complete selection summary before offering a write action

### Requirement: Preset descriptions match generated output
Interactive preset labels and descriptions SHALL describe only files and structure the selected preset currently generates and SHALL distinguish a preset from the framework stack.

#### Scenario: Company profile preset
- **WHEN** the preset menu displays Company Profile
- **THEN** it describes the generated page and layout structure without claiming complete SEO behavior

#### Scenario: Blog preset
- **WHEN** the preset menu displays Blog
- **THEN** it describes the MDX structure, sample post, and static RSS sample without calling it a complete content management system

#### Scenario: No preset
- **WHEN** the preset menu displays the empty preset
- **THEN** it explains that the selected stack still supplies runnable project files and only preset-specific files are omitted

### Requirement: User confirms generation and optional Git actions
Interactive generation SHALL offer preview, create, back, and cancel actions after the summary, and SHALL obtain explicit authorization before initializing or committing a Git repository.

#### Scenario: Preview before creation
- **WHEN** the user selects preview from the summary
- **THEN** the CLI shows planned files without changing the filesystem and returns to the summary

#### Scenario: Return to previous selection
- **WHEN** the user chooses back
- **THEN** the CLI returns to an earlier selection without writing project files

#### Scenario: Cancel at summary
- **WHEN** the user cancels at the summary
- **THEN** no project or Git state is created or modified

#### Scenario: Git automation not authorized
- **WHEN** the user creates the project but declines Git initialization or the initial commit
- **THEN** the generated project remains available without that declined Git side effect
