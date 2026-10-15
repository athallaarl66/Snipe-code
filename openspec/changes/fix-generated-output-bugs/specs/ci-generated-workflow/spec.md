## Purpose

Generated Node workflows use an install command compatible with the scaffold, which does not create a lockfile. This change does not claim that every generated lint, typecheck, build, or test step passes.

## ADDED Requirements

### Requirement: Install step is compatible with generated manifests
The generated `main.yml` workflow SHALL use `npm install` for a frontend or NestJS backend scaffold that does not write a lockfile, and SHALL NOT use `npm ci` for that path.

#### Scenario: Frontend scaffold without lockfile
- **WHEN** a project is generated with a frontend stack (nextjs, react-vite, vue, or nuxtjs)
- **THEN** the generated `.github/workflows/main.yml` frontend job contains `npm install` and does not contain `npm ci`

#### Scenario: NestJS backend without lockfile
- **WHEN** a project is generated with the `nestjs` backend
- **THEN** the generated backend job contains `npm install` and does not contain `npm ci`

### Requirement: Install fix is not a full CI pass claim
The workflow SHALL retain the existing stack-specific command structure, but this change SHALL NOT represent an unconditional lint or test command as successful when the generated manifest does not declare it.

#### Scenario: Manifest without a lint script
- **WHEN** a generated frontend manifest has no `lint` script
- **THEN** the install-command requirement remains satisfied, while lint validation remains an explicit deferred verification concern in `harden-generator-project-quality`

#### Scenario: Go backend workflow remains represented
- **WHEN** a project is generated with the `go-gin` backend
- **THEN** the generated backend job still contains its existing `go mod tidy` and `go build ./...` steps, without this change claiming native build success

> Scope boundary: capability-aware lint/typecheck/test steps, workflow script validation, and native stack matrix verification are deferred to `harden-generator-project-quality`.
