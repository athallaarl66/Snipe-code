## Context

See `proposal.md` for motivation. Product remains a single Rust package with a terminal UI and file-generation library. `GenerateConfig::project_name` currently doubles as a filesystem path, and `generate()` removes the destination before producing output. Templates, stack definitions, CI, Docker, AI workspace, and Git post-processing are currently separate writes, so later failures leave an already-published partial project.

Supported generated stacks are Next.js, React Vite, Vue, Nuxt, NestJS, Go Gin, Spring Boot, .NET 8, and Laravel, with PostgreSQL/MySQL options. Root repository tooling has no tracked CI, toolchain pin, or release workflow. The active `feat-stack-aware-audit` change owns stack rules but its root-only discovery and regex-only design do not cover generated `frontend/` and `backend/` layouts or non-regex checks. `fix-generated-output-bugs` is complete but has not been archived; this change supersedes its weaker output-integrity, CI, and report-completeness contracts where they conflict.

## Goals / Non-Goals

**Goals:**
- Preserve Rust-native CLI runtime and single-binary distribution.
- Make destination handling non-destructive and transactional from the user's perspective.
- Turn supported-stack promises into buildable, testable contracts.
- Run target toolchains only in isolated verification and release jobs.
- Provide stack-aware audit behavior honest about heuristic coverage.
- Give scripts deterministic exit semantics.
- Make release artifacts verifiable and documentation evidence-based.

**Non-Goals:**
- Add Node, Python, Go, Java, .NET, or PHP runtimes to the SNIPE-CODE binary.
- Turn scaffolded backends into complete production applications or prove application security.
- Add SAST based on AST/type analysis in this change.
- Preserve unsupported absolute or nested project-name inputs.
- Make every generated project byte-for-byte reproducible or publish a universal installer.
- Add dashboards, remote services, plugin systems, or database connections to SNIPE-CODE.

## Decisions

### D1: Keep Rust as the only runtime and embed stack data

`snipe-code` remains a Rust CLI/library. Existing embedded template/stack data remains the source for generation. Node, Go, Java, .NET, PHP, Composer, Docker, and Git remain external tools invoked only when generating integrations or verifying generated outputs.

Rationale: project is a local filesystem-oriented CLI; adding an embedded service runtime would increase binary size and distribution cost without filling any current contract.

Alternative considered: implement generator or verifier in Node/Python. Rejected because it replaces a working single-binary core with another runtime and does not solve output contracts.

### D2: Treat project name as one safe directory name

Validate `project_name` as a single, non-empty relative directory name before constructing any path. Reject path separators, absolute/root syntax, `.`/`..`, NUL, platform-reserved names, trailing dot/space, and names rejected by the host platform. Resolve against the current directory and canonicalize the parent for collision display. Do not accept nested destinations in this change.

Rationale: input is labeled “Project Name,” not “destination path.” A single component gives a small, testable trust boundary and avoids inventing a destination policy.

Alternative considered: permit relative nested paths with containment checks. Rejected for now; add when real workflow needs explicit parent-directory selection.

### D3: Stage output and make overwrite opt-in

Build all deterministic project content and post-generation files in a uniquely named sibling staging directory. Validate required files and generated CI before publication. New destinations publish by rename. Existing ordinary directories are removed only after explicit overwrite confirmation and successful staging. Existing final symlinks/junctions are always rejected. Cleanup only paths created by current run.

Post-generation steps are split into required and best-effort operations:

- Required content: template, stack files, `.env.example`, `.gitignore`, README, CI when selected stack requires it, Docker when selected, AI workspace when enabled.
- Best-effort external step: Git initialization/commit. Failure reports partial automation status but does not delete the successfully generated project.

Do not attempt full cross-platform atomic directory replacement. “Transactional” means the old destination survives generation failure and the visible destination is absent or complete at normal publication boundaries; a crash during final replacement may require recovery guidance.

Alternative considered: remove destination first as current code does. Rejected due to data-loss risk. Alternative considered: permanent backup plus copy. Rejected because it adds migration/cleanup policy without improving normal overwrite safety.

### D4: Model CI from declared capabilities

Generated workflows are assembled from stack-specific verified command sequences. Node jobs run `npm install`; they run `lint`, `typecheck`, and `test` only when package scripts exist. Every advertised stack still receives a build step. Backend jobs retain native restore/build/test commands after generated manifests are made complete.

Generated projects intentionally do not fabricate lockfiles. `npm install` remains valid; once a user creates a lockfile, generated CI may use `npm ci`. Verification checks workflow YAML and script/path existence before target builds.

Alternative considered: add ESLint/Jest/etc. to every scaffold automatically. Rejected because it inflates generated projects and hides missing declarations. Add a quality-tool stack only when explicitly requested.

### D5: Complete each stack with its native skeleton, not a universal template

Use each ecosystem's minimum conventional project shape:

- Vite: add `index.html`, mount markup, and required TS config references.
- Next/Nuxt: ensure app root, config, and generated type prerequisites build under declared versions.
- NestJS: retain its explicit module/bootstrap structure and complete TypeScript settings.
- Go: generated `go.mod` is authoritative; remove repeated `go mod init` instructions.
- Spring: use standard Maven source layout and package matching the application class.
- .NET: include the minimum web project metadata expected by the selected SDK.
- Laravel: include the framework bootstrap/application/kernel/route structure and autoload/script metadata required by `artisan` and tests; keep secrets in an environment template, not a committed real `.env`.

Docker backend support is enabled per stack only when a valid Dockerfile and runtime command exist. Unsupported backend containers are omitted; DB-only Compose remains valid. Compose database credentials are local-development placeholders and must be documented as replaceable, not production secrets.

Alternative considered: maintain one universal multi-language Dockerfile. Rejected. Alternative considered: emit a backend service and knowingly omit its Dockerfile. Rejected.

### D6: Add generated-project verification outside default unit tests

Fast Rust tests validate deterministic contracts: safe paths, staging behavior, JSON/YAML structure, workflow-script references, manifest completeness, and report/exit behavior. Heavy native verification runs through repository scripts invoked by root CI matrix jobs. It generates each stack in unique temp directories, installs dependencies, and runs documented build/test commands. Docker-enabled jobs validate Compose with Docker when relevant.

Tests use `std::env::temp_dir` plus collision-resistant names or an already-installed temp-directory helper; no new runtime is bundled. Native verification dependencies are declared in CI and docs, not linked into SNIPE-CODE.

Matrix minimum:

```text
frontend: nextjs | react-vite | vue | nuxt
backend:  nestjs | go-gin | spring-boot | dotnet8 | laravel
storage:  postgres | mysql
ci:       all selections
docker:   database-only + every backend declared Docker-capable
```

Every matrix job is required in release quality; no silent `continue-on-error` or optional skip.

### D7: Merge audit contracts before implementation

`feat-stack-aware-audit` becomes a prerequisite input to this change. First update its design/specs to discover contexts recursively, distinguish ecosystem from framework, include Rust, support file-presence/ignore/filename checks, preserve rule identity during dedupe, and disclose scope. Then implement it together with CLI result semantics here.

Use a typed rule model rather than pretending every check is regex:

- line-pattern rule: severity/category/description, file matcher, regex or literal token, optional negative context
- file-presence rule: required/forbidden path pattern and message
- ignore-containment rule: tracked file and ignore requirement
- config-content rule: filename and literal/regex checks

Manifest parsing is minimal and content-aware using existing `serde_json` for JSON manifests and simple framework markers for formats where adding a full XML/YAML parser is unnecessary. Do not call every `pom.xml` Spring or every `composer.json` Laravel. Add `regex` only if accepted by dependency review; stdlib literal matching remains preferred for fixed tokens.

Report text uses “No findings detected by evaluated rules,” not “Your codebase looks clean.” Each report lists contexts, rule groups, excluded paths, unknown scope, and heuristic coverage. Rust gets a minimum secret/unsafe-command rule set rather than being detected with no coverage.

Alternative considered: invoke Semgrep/tree-sitter externally for v1. Rejected because it creates a required external engine and conflicts with the single-binary/local promise. Record as future upgrade when rule accuracy requires syntax awareness.

### D8: Return structured audit results and map stable exit codes

Refactor command boundaries so prompts, generation, and audit return `Result`-like outcomes instead of printing and swallowing errors. `main()` owns process exit mapping:

```text
0  success, no policy failure
1  audit completed, findings meet failure threshold
2  invalid usage/unsupported export
3  runtime/read/write/export failure
```

Default audit failure threshold is HIGH. MEDIUM findings are reported but do not fail by default; make threshold an explicit CLI option only if product needs it. Unknown format validation happens before scanning or writing. Mixed known/unknown formats fail as usage. Duplicate formats are normalized once.

Audit result includes report paths, total and severity counts, highest severity, evaluated contexts/rule groups, and policy-failed boolean. Exporters receive one shared snapshot/date and cannot turn a policy failure into success.

### D9: Pin and sign releases through native CI primitives

Add `rust-toolchain.toml`, `rust-version`, and root CI. Required checks are `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-targets --all-features`, and generated-project matrix. Release uses standard Rust target triples for documented OS/architecture pairs.

Build release archives in native runners, generate SHA-256 checksum manifests, and sign/provenance them using GitHub OIDC plus Sigstore/cosign or GitHub artifact attestation. Select one mechanism during implementation after checking repository/account availability; do not ship a long-lived signing key. Release job fails if signing, checksum, or provenance generation is unavailable. Documentation includes exact verification commands.

Alternative considered: commit a private key. Rejected. Alternative considered: checksums only. Rejected because checksums detect corruption but do not establish artifact provenance.

### D10: Documentation claims map to checks

Replace “Enterprise-Grade” and “security-baked” with accurate “Rust CLI project scaffolder and heuristic security audit.” Build a small capability-to-evidence table in README: output contract, audit coverage, exit behavior, supported platforms, release verification. Remove unavailable release-channel promises until a signed release exists. Generated README states scaffold scope, optional/best-effort behavior, exact setup commands, and Docker support by stack.

Documentation updates follow implementation; tests assert critical literal claims do not regress (for example, no unsupported enterprise-grade label).

## Risks / Trade-offs

- [Native matrix is slow and network-dependent] → split by stack, cache target-language dependencies, parallelize jobs, retain only fast contracts in default `cargo test`.
- [Completing framework skeletons increases template size] → keep only files required to build/start each advertised framework; add optional quality tooling only on request.
- [Single-component project names reject current undocumented path use] → document breaking input restriction and error clearly; add nested destination support in a future change if needed.
- [Staging cannot make Windows directory replacement perfectly atomic] → never remove old destination before successful validation; add clear recovery output for interrupted final publication.
- [Regex audit remains imprecise] → report heuristic scope, preserve rule identity, use manifests for context, and set an upgrade trigger based on measured false positives or demand for syntax-aware analysis.
- [Recursive discovery can scan unrelated nested projects] → exclude VCS/dependency/build/output directories, cap traversal to recognized manifest locations, and report discovered contexts.
- [OIDC signing setup depends on release platform policy] → choose one documented attestation mechanism and fail closed; no unsigned fallback.
- [Existing active changes overlap and may conflict] → reconcile/archive `fix-generated-output-bugs` and revise `feat-stack-aware-audit` before application code changes begin.

## Migration Plan

1. Reconcile active OpenSpec changes: archive completed output-bug work only after code/test truth is corrected; update stack-aware audit specs with context/coverage contracts from this change.
2. Introduce shared result/error types and safe destination validation with destructive tests disabled by design.
3. Add staging/publication and unique temporary test workspaces; verify dry-run remains mutation-free.
4. Complete frontend/backend templates and remove mismatched setup commands.
5. Generate capability-aware CI and valid Docker output.
6. Add structural tests and native generated-project matrix.
7. Implement recursive stack contexts, typed rules, Rust coverage, honest reports, structured audit result, and exit-code mapping.
8. Add pinned toolchain, root CI, signed release workflow, checksum/provenance verification, and release docs.
9. Rewrite product claims and feature matrix from passing evidence.
10. Rollback: revert implementation commits; existing generated projects are not migrated. Failed staging is disposable; a user-confirmed replacement follows normal filesystem recovery if interrupted during final publish.

## Open Questions

None. Signature backend choice is implementation-time platform selection under one contract: OIDC-backed, keyless, and fail-closed. It does not change user-visible requirements or task boundaries.
