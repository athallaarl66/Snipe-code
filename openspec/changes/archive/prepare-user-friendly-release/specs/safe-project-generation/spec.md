## MODIFIED Requirements

### Requirement: Generation is staged before replacing an existing destination
The generator SHALL build required project output in an isolated sibling staging location, validate the staged result, and only then replace the requested destination. A failed generation or final publication SHALL remove only generator-owned temporary data and SHALL preserve or restore any pre-existing destination before reporting failure. If restoration itself fails, the generator SHALL retain the old project at a reported recovery path instead of deleting it.

#### Scenario: Write fails during staging
- **WHEN** any required generated file cannot be written or validated
- **THEN** the requested destination is not created or replaced and no partially generated project remains visible at that destination

#### Scenario: Successful new destination
- **WHEN** all required output is written and validated for a destination that does not exist
- **THEN** the completed output is published atomically at the requested destination

#### Scenario: Successful overwrite
- **WHEN** overwrite is authorized and staged output passes validation
- **THEN** the validated output replaces the existing destination and generator-owned temporary data is removed

#### Scenario: Final publication fails after overwrite approval
- **WHEN** the old destination has been moved aside and publishing the staged output fails
- **THEN** the old project is restored at its destination or retained at a clearly reported recovery path, and the failure is returned

#### Scenario: Successful replacement cleanup fails
- **WHEN** the new project has been published but cleanup of the old project fails
- **THEN** the command reports the retained old-project path and does not silently delete user data or claim clean completion
