## ADDED Requirements

### Requirement: Template struct definition
The system SHALL define a `Template` struct with fields: `id` (String), `name` (String), `description` (String), `icon` (String), `structure` (TemplateStructure).

#### Scenario: Template struct can be instantiated
- **WHEN** a Template is created with valid fields
- **THEN** all fields are accessible and contain correct values

### Requirement: TemplateStructure definition
The system SHALL define a `TemplateStructure` struct with fields: `folders` (Vec<String>), `files` (Vec<FileDef>), `rules` (GenerationRules).

#### Scenario: TemplateStructure holds folder and file definitions
- **WHEN** a TemplateStructure is created
- **THEN** folders and files are accessible as vectors

### Requirement: FileDef definition
The system SHALL define a `FileDef` struct with fields: `path` (String), `template_source` (String), `condition` (Option<String>).

#### Scenario: FileDef with condition
- **WHEN** a FileDef has a condition value
- **THEN** condition is Some("...")

#### Scenario: FileDef without condition
- **WHEN** a FileDef has no condition
- **THEN** condition is None

### Requirement: GenerationRules definition
The system SHALL define a `GenerationRules` struct with fields: `require_backend` (bool), `require_database` (bool), `docker_compose` (bool), `security_configs` (bool).

#### Scenario: Default GenerationRules
- **WHEN** GenerationRules is created with default values
- **THEN** all booleans are false

### Requirement: TemplateRegistry load all templates
The system SHALL provide a `TemplateRegistry::load_all()` function that returns a Vec of all template definitions.

#### Scenario: All templates are loaded
- **WHEN** TemplateRegistry::load_all() is called
- **THEN** it returns exactly 4 templates (none, company-profile, portfolio, blog)

#### Scenario: Template "none" is first
- **WHEN** templates are loaded
- **THEN** the first template has id "none"

### Requirement: TemplateRegistry get by ID
The system SHALL provide a `TemplateRegistry::get(id: &str)` function that returns a specific template.

#### Scenario: Get existing template
- **WHEN** get("company-profile") is called
- **THEN** it returns Some(Template) with id "company-profile"

#### Scenario: Get non-existing template
- **WHEN** get("nonexistent") is called
- **THEN** it returns None

### Requirement: Template JSON deserialization
The system SHALL deserialize template definitions from embedded JSON strings.

#### Scenario: None template has empty structure
- **WHEN** none template is loaded
- **THEN** folders and files are empty vectors

### Requirement: Template has required fields
Every template SHALL have non-empty `name`, `description`, and `icon` fields.

#### Scenario: All templates have non-empty metadata
- **WHEN** all templates are loaded
- **THEN** every template has name, description, and icon that are not empty strings
