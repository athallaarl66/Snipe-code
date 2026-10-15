# Spec Delta

## Purpose

Defines the heuristic scan of a project folder and the reports it exports, including the metadata every report carries and the disclosure of what was and was not examined.

## ADDED Requirements

### Requirement: Exported reports record the actual generation date
Every exported audit report MUST carry the real system date at which the report was generated, never a fixed or placeholder value.

#### Scenario: Report date reflects the run
- **WHEN** an audit report is exported on a given date
- **THEN** the report shows that date

#### Scenario: No fixed date is emitted
- **WHEN** audit reports are exported on two different dates
- **THEN** the two reports show different dates

### Requirement: All export formats carry the same report metadata
Every supported export format MUST present the same generation date, detected contexts, and finding totals for a single audit run.

#### Scenario: Formats agree
- **WHEN** one audit run exports Markdown, PDF, and DOCX
- **THEN** all three report the same generation date, detected contexts, and finding counts
