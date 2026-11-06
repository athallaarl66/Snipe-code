## ADDED Requirements

### Requirement: Heuristic HIGH findings avoid known benign assignments
The audit SHALL NOT classify a source line as a hardcoded secret or unsafe query solely because it contains a rule keyword and an assignment or plus sign. Narrowing a pattern SHALL retain detection of fixture-backed unsafe examples and keep the result labeled heuristic.

#### Scenario: Benign source text
- **WHEN** an assignment contains a variable name with `token` or a query label but no embedded credential or concatenated executable query
- **THEN** the audit does not emit a HIGH finding merely from the keyword

#### Scenario: Unsafe source text
- **WHEN** a supported source fixture contains an actual literal credential assignment or concatenated SQL query
- **THEN** the corresponding HIGH finding remains present with a source location
