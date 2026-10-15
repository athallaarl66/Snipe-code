## Purpose

This change narrows scaffold integrity to files it actually checks. It makes the selected NestJS JSON file parseable, provides a Laravel environment sample with a non-empty key placeholder, and removes the known duplicate ignore entry. It does not claim that any generated framework project boots or passes its native test command.

## ADDED Requirements

### Requirement: Selected generated JSON file is parseable
The `nest-cli.json` written for the NestJS backend SHALL be a well-formed JSON object containing `$schema` and `sourceRoot`.

#### Scenario: NestJS project generation
- **WHEN** a project is generated with the `nestjs` backend stack
- **THEN** the generated `backend/nest-cli.json` parses as JSON and contains the expected schema and source-root keys

#### Scenario: Frontend package file remains parseable
- **WHEN** a project is generated with a frontend stack
- **THEN** the generated `frontend/package.json` is parseable JSON, without this change claiming its build succeeds

### Requirement: Laravel environment sample contains a non-empty placeholder
When a project is generated with the `laravel` backend, the scaffold SHALL include `backend/.env` with a non-empty `APP_KEY` placeholder value.

#### Scenario: Laravel environment sample
- **WHEN** a project is generated with the `laravel` backend stack
- **THEN** the generated `backend/.env` contains an `APP_KEY=` line whose value is non-empty

#### Scenario: Bootability is not inferred
- **WHEN** only the environment sample is present
- **THEN** this change does not claim that Laravel bootstrap, Artisan, or `php artisan test` succeeds without the complete skeleton and native verification

### Requirement: Generated gitignore has no duplicate vendor entry
The `.gitignore` written by the generator SHALL contain `vendor/` at most once.

#### Scenario: Generate project with gitignore
- **WHEN** a project is generated
- **THEN** its `.gitignore` contains one `vendor/` entry, not a duplicate

> Scope boundary: complete frontend/backend skeletons, Composer metadata, runnable Laravel contracts, native build/test matrices, and broader generated-file validity are deferred to `harden-generator-project-quality`.
