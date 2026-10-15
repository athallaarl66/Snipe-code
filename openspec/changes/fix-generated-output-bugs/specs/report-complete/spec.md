## Purpose

The PDF exporter discloses findings omitted by its current one-page layout. This is a truncation signal, not a claim that the PDF is paginated or that every finding is represented in every report format.

## ADDED Requirements

### Requirement: PDF report discloses omitted findings
When the PDF exporter reaches its page-space limit before printing all findings, the report SHALL print a closing line stating the number of findings that were not listed.

#### Scenario: Findings exceed one page
- **WHEN** audit findings are too many to fit the current PDF page space
- **THEN** the PDF contains a line stating how many findings were omitted, such as `3 more findings omitted — page space exceeded`

### Requirement: Findings printed before the limit remain present
Up to the page limit, findings SHALL retain the existing title, file, line, and description layout.

#### Scenario: Findings fit the page
- **WHEN** all findings fit the available page space
- **THEN** every finding is listed and no omission line is printed

> Scope boundary: multi-page PDF generation, complete pagination, DOCX parity, and end-to-end representation of every finding are deferred to `harden-generator-project-quality`.
