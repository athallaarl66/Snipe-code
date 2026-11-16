## Purpose

Hardcoded-secret scan menambahkan dua nama dotfile eksplisit, `.env` dan `.env.local`, pada recursive scan. Requirement ini tidak mengklaim semua dotfile atau seluruh pola secret tercakup.

## ADDED Requirements

### Requirement: Secret scan includes supported dotfile names
The audit's secret-hardcoded check SHALL inspect `.env` and `.env.local` recursively when they occur outside excluded directories, in addition to files with a configured source extension.

#### Scenario: Hardcoded secret in .env
- **WHEN** audit runs on a folder containing `.env` with a line like `API_KEY=abc123`
- **THEN** the report includes a Hardcoded Secret finding pointing at that `.env` file with the correct line number

#### Scenario: Hardcoded secret in .env.local
- **WHEN** audit runs on a folder containing `.env.local` with a secret-named assignment
- **THEN** the report includes a Hardcoded Secret finding for that file and line

#### Scenario: No secret in supported dotfile
- **WHEN** audit runs on a folder containing `.env` or `.env.local` whose assignments use placeholder values (`your-...`, `process.env...`)
- **THEN** no Hardcoded Secret finding is produced for that line

### Requirement: Existing extension and directory exclusions are preserved
Files with matching source extensions (`.ts`, `.tsx`, `.js`, `.jsx`, `.json`) and the existing hidden-directory, `node_modules`, and `target` exclusions SHALL keep working as before; the two-name dotfile addition is additive.

#### Scenario: Existing extension scan
- **WHEN** audit runs on a folder containing `.ts` with a hardcoded secret
- **THEN** the finding is produced as before the change

#### Scenario: Hidden directories still skipped
- **WHEN** audit runs on a folder containing a hidden directory or `node_modules/` with matching files
- **THEN** those files are skipped and are not presented as evaluated dotfile scope

#### Scenario: Dotfile breadth is not implied
- **WHEN** audit runs on a folder containing an unrelated dotfile such as `.env.example`
- **THEN** this change does not claim that the file was included in the secret line scan unless another implemented rule covers it

> Scope boundary: broader dotfile discovery, `.env.example` line inspection, and additional secret syntaxes are deferred to `feat-stack-aware-audit` and `harden-generator-project-quality`.
