## 1. Reconcile Existing Changes and Test Harness

- [x] 1.1 Re-audit `fix-generated-output-bugs` against current source; correct its OpenSpec claims for Laravel bootability, generated CI lint/test steps, PDF omission, dotfile scope, and real date verification before archiving
- [x] 1.2 Update `feat-stack-aware-audit` design/specs to cover recursive `frontend/` and `backend/` discovery, ecosystem-versus-framework identity, Rust rules, typed non-regex checks, rule-ID deduplication, and honest scope reporting
- [x] 1.3 Add `rust-toolchain.toml` with the selected stable toolchain and add `rust-version` matching supported policy to `Cargo.toml`
- [x] 1.4 Replace fixed-path generation tests with unique temporary workspaces and add regression tests proving repeated/parallel runs cannot delete repository fixtures

## 2. Safe Project Destinations and Staged Generation

- [x] 2.1 Add one project-name validator covering empty input, separators, absolute/root syntax, `.`/`..`, NUL, reserved names, and platform-invalid trailing characters
- [x] 2.2 Resolve and validate the destination parent before any write; return typed errors for invalid path and unsafe final symlink/junction destinations
- [x] 2.3 Refactor `generate()` to create a unique sibling staging directory, write required template/stack/security/CI/Docker/AI output there, and validate staged contracts before publication
- [x] 2.4 Publish new destinations by rename and replace existing ordinary directories only after explicit overwrite authorization; clean only current-run staging paths on success or failure
- [x] 2.5 Refactor wizard collision handling to show canonical destination, return cancellation without mutation, and propagate generation failure to process status
- [x] 2.6 Make dry-run path validation and preview free of filesystem creation, deletion, replacement, and writes
- [x] 2.7 Add tests for unsafe names, roots, traversal, symlink/junction targets, canceled overwrite, failed staging rollback, successful overwrite publication, and mutation-free dry run

## 3. Complete Frontend Scaffolds

- [x] 3.1 Complete React Vite output with `index.html`, matching root mount element, and valid TypeScript/Vite configuration; add build verification
- [x] 3.2 Complete Vue output with `index.html`, matching `#app` mount, Vue entry, and resolvable TypeScript/Vite configuration; add typecheck/build verification
- [x] 3.3 Verify and minimally repair Next.js App Router and TypeScript prerequisites under its declared Node/Next versions
- [x] 3.4 Verify and minimally repair Nuxt application root, configuration, generated-type prerequisites, and production build
- [x] 3.5 Add structural tests asserting every framework-required entry/config file exists and every generated package build script is declared

## 4. Complete Backend Scaffolds

- [x] 4.1 Complete NestJS TypeScript/Nest configuration and verify `npm run build` plus a minimal startup/build contract
- [x] 4.2 Make Go `go.mod` authoritative, remove repeated `go mod init` guidance, and verify `go test ./...` plus `go build ./...`
- [x] 4.3 Move Spring application source to conventional Maven package path with matching package declaration and verify `mvn test package`
- [x] 4.4 Complete .NET 8 web project metadata and verify restore/build under the declared SDK
- [x] 4.5 Complete Laravel Composer autoload/scripts, Artisan bootstrap/application/kernel/exception/routes, environment template, and minimal tests; verify `composer install` and `php artisan test`
- [x] 4.6 Remove generated real-secret `.env` practices in favor of safe environment templates and documented local/CI setup

## 5. Capability-Aware Generated CI and Docker

- [x] 5.1 Refactor generated CI per stack so Node lint/typecheck/test steps run only when matching package scripts exist, while every advertised frontend/backend receives a valid build step
- [x] 5.2 Add explicit test steps for generated backends with executable native test commands and remove nonexistent commands from other workflows
- [x] 5.3 Remove `npm init -y` or manifest-recreation instructions when generator already wrote `package.json`; align README setup commands with generated files
- [x] 5.4 Declare Docker support per backend; add valid Dockerfile/runtime files only for supported backends and omit backend services without build support
- [x] 5.5 Make database-only Compose valid, use replaceable local-development credential placeholders, and document non-production credential scope
- [x] 5.6 Add structural tests that parse generated JSON/YAML, check referenced paths, and compare every Node CI script with `package.json`

## 6. Native Generated-Project Verification

- [x] 6.1 Add a cross-platform verification entry point that generates a selected stack into an isolated workspace and runs its documented install/build/test commands
- [x] 6.2 Add required frontend matrix jobs for Next.js, React Vite, Vue, and Nuxt with pinned target-language versions and dependency caching
- [x] 6.3 Add required backend matrix jobs for NestJS, Go Gin, Spring Boot, .NET 8, and Laravel with pinned target-language versions and dependency caching
- [x] 6.4 Add required PostgreSQL/MySQL and supported-backend Compose validation jobs using Docker; fail when Docker or a declared build context is unavailable
- [x] 6.5 Ensure matrix jobs clean only their own workspaces and publish concise failure logs naming stack, command, destination, and exit code

## 7. Stack-Aware Audit Model and Discovery

- [x] 7.1 Replace root-only detection with bounded recursive project-context discovery, including generated `frontend/` and `backend/` roots and exclusion of VCS/dependency/build/report directories without following directory symlinks
- [x] 7.2 Add content-aware ecosystem/framework identity for JS/TS, React, Vue, Nuxt, PHP, Laravel, Go, Java, Spring, .NET, and Rust; classify ambiguous/misleading manifests honestly
- [x] 7.3 Define typed line-pattern, file-presence, ignore-containment, and config-content rules with filenames/extensions, negative context, stable IDs, severities, and descriptions
- [x] 7.4 Add common and meaningful rules for JS/TS, PHP/Laravel, Go, Java/Spring, .NET, and Rust; ensure every detected supported stack executes a non-empty applicable rule set
- [x] 7.5 Scope rules to their owning project context and deduplicate only identical rule instances while retaining distinct rules on the same file/line
- [x] 7.6 Add tests for nested contexts, monorepos, misleading manifests, framework distinction, all supported stacks, unknown/partial scope, non-regex checks, and rule-identity deduplication

## 8. Audit Results, Reports, and Exit Codes

- [x] 8.1 Return one audit result containing report paths, total/severity counts, highest severity, detected scopes, evaluated rule groups, exclusions, and policy-failed state
- [x] 8.2 Validate and normalize export formats before scanning; reject empty/unknown/mixed tokens and normalize duplicates without silent writes
- [x] 8.3 Refactor command entry points to propagate typed errors and map process exits to `0` success, `1` HIGH-finding policy failure, `2` usage error, and `3` runtime/export error
- [x] 8.4 Write operational, usage, and policy failures to standard error; keep success/report paths on standard output
- [x] 8.5 Update Markdown, PDF, and DOCX output to share one audit snapshot/date, list evaluated/unknown scope, use “No findings detected by evaluated rules,” and represent or disclose every finding
- [x] 8.6 Add command-level tests for success, HIGH finding, LOW-only finding, missing export value, unknown/mixed/duplicate formats, missing path, report write failure, stderr routing, and generated CI policy use

## 9. Root CI and Release Assurance

- [x] 9.1 Add root CI running pinned-toolchain reporting, `cargo fmt --check`, Clippy with warnings denied, full Rust tests, and all required generated-project matrix jobs
- [x] 9.2 Add minimum-Rust-version verification using the stable compiler available in CI
- [x] 9.3 Add release workflow using valid Rust target triples for every documented supported OS/architecture and native archive packaging
- [x] 9.4 Exclude build caches and test data from release archives; include executable, applicable extension, license, and README
- [x] 9.5 Add SHA-256 checksum manifests and one keyless OIDC-backed signature/provenance mechanism selected for repository release hosting
- [x] 9.6 Fail release publication when checksum, signing, provenance, required tests, or artifact inspection is missing
- [x] 9.7 Add release tests or workflow validation proving artifact names, checksums, signature/provenance, and source revision are mutually verifiable

## 10. Documentation Alignment

- [x] 10.1 Replace unsupported “enterprise-grade,” “security-baked,” clean-codebase, and unavailable-release claims with accurate Rust CLI, scaffold, and heuristic-audit language
- [x] 10.2 Publish a supported stack/platform matrix mapping each advertised capability to its automated verification job
- [x] 10.3 Document safe single-name project destinations, overwrite/staging behavior, best-effort Git automation, and recovery expectations
- [x] 10.4 Document exact generated setup, test, Docker support, audit scope/limitations, exit codes, and report interpretation per stack
- [x] 10.5 Document exact install, download, checksum, signature/provenance, and source-revision verification commands only for artifacts actually published
- [x] 10.6 Add documentation tests/checks preventing unsupported quality labels, stale test-count claims, nonexistent commands, and drift between README and generated stack capabilities

## 11. Final Verification and OpenSpec Reconciliation

- [x] 11.1 Run `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test --all-targets --all-features`
- [x] 11.2 Run complete frontend, backend, database, Docker, workflow, path-safety, audit, and exit-code verification with durable CI logs/artifact summaries
- [x] 11.3 Generate representative full-stack projects and confirm clean install/build/test plus valid CI and Docker output for each supported capability
- [ ] 11.4 Run one release rehearsal and verify archive contents, checksums, signature/provenance, and source revision from a clean checkout
- [x] 11.5 Re-check README/PRD/technical documentation against passing evidence and remove or downgrade every unsupported claim
- [x] 11.6 Update/archive superseded OpenSpec changes and sync this change’s delta specs into canonical specs without losing stricter requirements
