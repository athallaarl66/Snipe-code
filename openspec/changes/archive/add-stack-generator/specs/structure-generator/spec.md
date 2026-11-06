## ADDED Requirements

### Requirement: Generate folders from stack selection
The system SHALL create folder structure based on selected frontend and backend stacks.

#### Scenario: Frontend only
- **WHEN** frontend is Next.js AND backend is None
- **THEN** creates frontend folders only (components/ui, components/layout, pages, hooks, lib)

#### Scenario: Backend only
- **WHEN** frontend is None AND backend is NestJS
- **THEN** creates backend folders only (controllers, services, repositories, middleware, routes)

#### Scenario: Both selected
- **WHEN** frontend is Next.js AND backend is NestJS
- **THEN** creates both frontend and backend folders

### Requirement: Generate security config files
The system SHALL generate `.env.example` and `.gitignore` in the project root.

#### Scenario: Security files created
- **WHEN** project is generated
- **THEN** `.env.example` exists with placeholder values
- **THEN** `.gitignore` exists with standard exclusions

### Requirement: Generate README
The system SHALL generate a `README.md` with project name and stack info.

#### Scenario: README created
- **WHEN** project is generated
- **THEN** `README.md` exists with project name

### Requirement: Project name validation
The system SHALL validate project name is not empty and contains only valid characters.

#### Scenario: Empty name rejected
- **WHEN** project name is empty
- **THEN** generation fails with error

#### Scenario: Valid name accepted
- **WHEN** project name is "my-project"
- **THEN** generation proceeds

### Requirement: Output path
The system SHALL generate all files in a directory named after the project.

#### Scenario: Correct output path
- **WHEN** project name is "my-app"
- **THEN** all files are created under `my-app/`
