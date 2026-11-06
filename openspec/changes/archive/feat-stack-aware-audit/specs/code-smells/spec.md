## Purpose

Basic code smells remain deterministic heuristics, but only evaluated files produce findings. Excluded and unknown paths stay visible in report scope.

## ADDED Requirements

### Requirement: Code smell detection
The system SHALL detect basic code smells across evaluated source files of detected contexts, without AI or external services.

#### Scenario: Overlong function
- **WHEN** a supported function body exceeds 50 lines using its language structure
- **THEN** a LOW heuristic finding is reported with file and starting line

#### Scenario: Deep nesting
- **WHEN** indentation nesting exceeds 3 levels inside a supported function
- **THEN** a LOW heuristic finding is reported

#### Scenario: Magic numbers
- **WHEN** a numeric literal appears in logic more than once without a named constant
- **THEN** a LOW heuristic finding is reported

#### Scenario: TODO/FIXME/HACK markers
- **WHEN** an evaluated source file contains `TODO`, `FIXME`, or `HACK`
- **THEN** an INFO finding is reported with file and line

#### Scenario: Empty catch block
- **WHEN** a catch/except block contains no statements and only whitespace or comments
- **THEN** a MEDIUM heuristic finding is reported

#### Scenario: Assignment in condition
- **WHEN** an assignment (`=`) appears inside a condition expression (`if (x = 5)`)
- **THEN** a HIGH finding is reported

#### Scenario: Duplicate string literals
- **WHEN** the same string literal appears more than 3 times within one evaluated file
- **THEN** a LOW finding is reported

### Requirement: Smell scope is bounded and disclosed
Smell checks SHALL run only on files owned by an evaluated context. Hidden directories, VCS/dependency/build/report directories, and symlink paths SHALL be excluded and reported; unknown contexts SHALL not be described as smell-free.

#### Scenario: Vendored and binary paths skipped
- **WHEN** scanning recursively encounters `node_modules`, `target`, `vendor`, `bin`, `obj`, build/output, hidden, or symlink paths
- **THEN** those paths are excluded from smell evaluation and listed with reasons

#### Scenario: Unknown context wording
- **WHEN** a directory has no recognized manifest
- **THEN** the report identifies smell coverage as unknown rather than claiming all files passed

### Requirement: Heuristic limitations are disclosed
The system SHALL label code-smell findings as heuristic and SHALL not present them as complete static-analysis proof.

#### Scenario: Report limitation
- **WHEN** a code-smell report is generated
- **THEN** the report states that smell checks cover only evaluated files and may produce false positives or miss constructs
