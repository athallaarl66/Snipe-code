# Tasks

## 1. QA plan contract

- [x] 1.1 Define test-case model and requirement/source trace IDs; verify schema fixtures for positive, negative, boundary, authorization, validation, and regression cases.
- [x] 1.2 Add read-only `qa plan` input path selection; use filesystem snapshots in tests to prove no project writes, child processes, endpoint requests, or tool installs occur.
- [x] 1.3 Add explicit-destination plan export; verify default planning writes nothing and invalid destination reports failure without partial-success status.

## 2. Generated script review

- [x] 2.1 Choose first adapter only after confirming supported framework and tool version; document adapter version and required environment with fixtures.
- [x] 2.2 Generate draft script and run static syntax/structure validation; verify unsupported adapters are rejected and valid scripts remain `ready-for-review`, never `passed`.
- [x] 2.3 Add unit/contract test mapping case ID to generated script and expected result; verify generated script has no implicit install or live request.

## 3. Runner policy before platform execution

- [x] 3.1 Define immutable run manifest with command, cwd, target, redacted env names, mounts, network, output, timeout, and install effects; test manifest hash changes invalidate prior approval.
- [x] 3.2 Implement fake-runner policy tests for missing consent, stale approval, ambiguous/remote/production DB, no-config disclosure requiring explicit approval, and deceptive `test` name; assert rejected runs spawn zero commands.
- [ ] 3.3 Document actual execution boundary per OS; verify reports do not call host execution or temporary workspace a sandbox and enumerate filesystem/network/endpoint/hardcoded-target limits.
- [ ] 3.4 Implement the first approval-bound runner mode only after its command, target, and DB policy are defined; clear recognized inherited DB host/target variables, block explicit remote/production-like/ambiguous DB settings, warn and require CLI approval when DB configuration is not detected, and test policy with fake runners. Add an OS sandbox adapter only as a separately evidenced capability; never imply it from temp paths.
- [ ] 3.5 Record actual command exit, artifacts, skipped/unavailable/blocked status; tests verify no unavailable, timeout, or blocked case is reported passed.

## 4. Documentation and gates

- [ ] 4.1 Document planning-only default, reviewed consent, DB restrictions, OS capability limits and legacy `verify` effects; verify examples make no unsupported sandbox claim.
- [x] 4.2 Run unit/contract tests for plan and policy without real target processes; fake runners suffice for local policy contracts. If a separately scoped integration test accesses a DB or claims sandboxing, use a disposable DB and verify the actual OS boundary; obtain separate approval. Do not require Docker, external CI, or human UAT to validate local planning/policy contracts.

## 5. CLI, agent, skills, and optional integrations

- [x] 5.1 Define CLI-only workflows and agent parity using shared policy and manifests; add testing skill for user projects without implicit install, execution, or network access.
- [ ] 5.2 Define optional local/online AI and MCP configuration; verify planning works without them and preview plus explicit user approval precedes endpoint access and each scoped data transmission, including local integrations.
- [ ] 5.3 Verify install, test, endpoint access, and data transmission each require an explicit preview and user approval bound to exact effects; deny missing, stale, or agent-only approval.

## 6. Phased adapters and OS acceptance

- [ ] 6.1 Define and verify unit/regression pilot adapter and selected framework/version before expanding support.
- [x] 6.2 Add phased adapter contracts and fixtures for Playwright, Postman collections using Postman CLI, Bruno, and Maestro; keep each unsupported until its own validation and execution boundary are ready.
- [ ] 6.3 Add CLI/agent contract tests for planning, preview, consent, changed scope and results; report untested OS boundaries and human UAT separately without claiming parity.

## 7. Postman setup and integration boundaries

- [ ] 7.1 Pilot Postman CLI first: detect installed version/configuration and preview trusted install source, destination, collection, environment, endpoint, and data before separate setup approval. Missing tool is unavailable; denied approval is blocked; never persist plaintext credentials.
- [ ] 7.2 Define evidence and limits for any future sandbox mode: disposable DB identity, restricted credentials, network policy, fixtures, reset, and cleanup boundary. Do not make these external services prerequisites for local policy tests. Host mode reports actual non-isolated filesystem/process/network/secret exposure and requires exact CLI approval; explicit remote/production-like/ambiguous DB remains blocked, while undetected DB config warns and requires approval.
- [x] 7.3 Keep M5 provider runtime and service-specific connectors outside current adapter delivery until scope, credential storage, retention, payload consent, idempotency, and permission contracts are approved; basic offline planning stays usable without models.

See `../../CR-ALIGNMENT.md`.

## Verified implementation and pending decisions

- Final integrated source state, local Windows, rustc 1.85.0: `cargo fmt --check`, `cargo clippy --all-targets --all-features --locked -- -D warnings`, and `cargo test --all-targets --all-features --locked` exit 0. Complete log: `C:/Users/Athalla arli/AppData/Local/hermes/cache/scratch/snipe-qa-final-resume.log`, `GATE_EXIT=0`; 262 passed, 0 failed, 0 ignored across 31 target summaries. This includes the Postman draft, fake policy, agent skill, and CRLF parser edits. No Linux/macOS or external CI run is claimed.
- Tasks 1.1-1.3 and 3.1 verified. Planning snapshot retains file bytes and directory entries; local listener receives no request, empty PATH prevents tool lookup, project scripts remain inert. Source inspection complements runtime canaries for process/network absence; no OS sandbox claim.
- Tasks 2.1-2.3 and 6.2: structured HTTP request fields selected by user; Collection v2.1 generation targets Postman CLI 1.90.0. Official version/format sources are recorded in design. Fixture verifies method, URL, headers, raw body, expected-status assertion, trace ID, unsupported adapter, invalid input, missing request, credential-header rejection, explicit export, and clear rejection of unit/regression, Playwright, Bruno, and Maestro adapter requests. `node --check` exit 0 for both generated assertion scripts; this proves syntax only, not Postman runtime compatibility. Postman CLI remains absent and was not installed.
- Tasks 3.2 and 4.2: `cargo test --locked --test qa_plan_test --test qa_runner_test` and `cargo test --locked --lib qa_runner::tests` exit 0: 5 planning, 4 runner integration, 4 runner unit tests. Fake callback is never invoked for rejected policy/approval/sandbox cases. Approved fake observations cover all result states; all untrusted logs and artifact paths are omitted as `[REDACTED]`. This is not effective-configuration preflight, actual process execution, artifact creation, timeout enforcement, or environment scrubbing; tasks 3.4-3.5 remain unchecked.
- Tasks 5.1 and 7.3: opt-in `snipe-qa-planning` artifacts reuse the existing renderer for Claude Code, OpenCode, Hermes Agent, and Antigravity. `qa_agent_test` has 4 passing tests: supported-host inventory, rendered approval/read-only contract, project-local opt-in generation, and executable CLI/library planning equivalence. Existing `ai_integration_test` has 17 passing tests. Agent execution/approval enforcement and OS parity are not delivered by these instruction files; tasks 5.3 and 6.3 remain unchecked. AI/MCP provider runtime and connectors remain absent; basic planning works without them.
- No target application test, DB connection, tool install, model/MCP request, sandbox evidence, or human UAT was performed. `node --check` was syntax-only. Local Cargo tests execute the product CLI and isolated developer fixtures, not target-project scripts.
- Existing quality documentation contract test required CRLF normalization on Windows; focused regression and full suite passed after the one-line test-only fix.

## Remaining gates

- 3.3, 4.1, 7.2: formal documentation/OS-boundary handoff to the technical-writer profile; generated skill text is not a substitute for the complete delivery documentation.
- 3.4: design decision pending before host execution. Current manifests bind caller-redacted descriptive strings, not actual executable/arguments, collection/environment bytes, effective DB settings, and Postman reporting configuration. The terminal approval API does not establish a human-controlled channel against automated PTYs. Define and verify this execution/consent binding before connecting a host runner; this blocker belongs to `qa-runner` in this change, not the history capability.
- 3.5: fake result-state/redaction matrix passes, but actual command exits, timeout/cancel enforcement, environment clearing, and produced artifacts require the host runner.
- 5.2, 5.3, 6.3: offline planning and generated instruction contracts pass. Optional integration configuration and enforced runtime consent/parity are not implemented; instructions and fake callbacks alone do not prove these tasks.
- 6.1, 6.2: only the Postman static draft slice is validated. A subsequent unit/regression framework/version and the remaining adapter contracts/fixtures are not selected or verified.
- 7.1: Postman CLI is absent; no setup/install or runtime compatibility gate ran. Setup source, destination, configuration/reporting effects and separate user approval remain required. Tool absence does not block the already-passing local planning/policy tests.

## Workflow follow-up

- The planning-only capability may be applied before a runner implementation is approved.
- Planning and fake-runner policy checks may proceed without external CI, Docker, DB, or human UAT. Host execution requires explicit scope approval, DB preflight, and honest non-isolated reporting; it is not a sandbox fallback. Sandbox-mode execution remains blocked until the requested OS boundary is proven.
- Consume the result schema from `add-scan-history` when linking plan/run records; apply that change first.
