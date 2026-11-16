## ADDED Requirements

### Requirement: Frontend stack definitions
The system SHALL define frontend stack options: Next.js, React (Vite), Vue, Nuxt.js, and None.

#### Scenario: All frontend options available
- **WHEN** frontend stacks are loaded
- **THEN** there are exactly 5 options including "None"

#### Scenario: Each frontend stack has folder structure
- **WHEN** a frontend stack is selected (not None)
- **THEN** it defines folders: `src/components/ui/`, `src/components/layout/`, `src/pages/`, `src/hooks/`, `src/lib/`

### Requirement: Backend stack definitions
The system SHALL define backend stack options: .NET 8, NestJS, Go (Gin), Spring Boot, Laravel, and None.

#### Scenario: All backend options available
- **WHEN** backend stacks are loaded
- **THEN** there are exactly 6 options including "None"

#### Scenario: Each backend stack has folder structure
- **WHEN** a backend stack is selected (not None)
- **THEN** it defines folders: `src/controllers/`, `src/services/`, `src/repositories/`, `src/middleware/`, `src/routes/`

### Requirement: Database stack definitions
The system SHALL define database options: PostgreSQL, MySQL, and None.

#### Scenario: All database options available
- **WHEN** database stacks are loaded
- **THEN** there are exactly 3 options including "None"

### Requirement: Stack selection validation
The system SHALL validate that at least one of frontend or backend is selected.

#### Scenario: Both None selected
- **WHEN** frontend is None AND backend is None
- **THEN** validation fails with error message

#### Scenario: At least one selected
- **WHEN** frontend is not None OR backend is not None
- **THEN** validation passes
