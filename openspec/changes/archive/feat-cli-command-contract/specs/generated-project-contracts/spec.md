## ADDED Requirements

### Requirement: Generated ignore rules cover tool-created artifacts

The `.gitignore` written into a generated project SHALL ignore the build output and report files that the generated workflows and this CLI's own commands create inside that project, including `bin/`, `obj/`, and the three `Audit_Report.*` report names. Lockfiles SHALL NOT be ignored, because they belong in the repository.

#### Scenario: Audit report inside a generated project
- **WHEN** a user runs `snipe-code audit . --export md` inside a generated project, as the generated `/snipe-code` command instructs
- **THEN** `Audit_Report.md` is not listed as an untracked or modified file by `git status`

#### Scenario: .NET build output
- **WHEN** a generated project uses the .NET backend and its declared build runs
- **THEN** `bin/` and `obj/` are not listed as untracked files by `git status`

#### Scenario: Lockfile stays tracked
- **WHEN** a generated project's `npm install` produces `package-lock.json`
- **THEN** the file remains visible to `git status`, so the lockfile can be committed
