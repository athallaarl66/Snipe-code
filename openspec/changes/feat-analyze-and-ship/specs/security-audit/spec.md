# Spec Delta

## ADDED Requirements

### Requirement: Audit findings are available as structured data
An audit run MUST be able to emit its detected contexts, each finding with its severity and location, and the finding totals as structured machine-readable data.

#### Scenario: Structured findings
- **WHEN** an audit run requests structured output
- **THEN** it emits the detected contexts, each finding with its severity and location, and the totals as structured data

#### Scenario: Structured output agrees with the exported report
- **WHEN** one audit run produces both structured output and an exported report
- **THEN** the contexts, findings, and totals agree between the two
