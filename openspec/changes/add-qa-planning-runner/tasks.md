# Tasks

## 1. QA plan contract

- [ ] 1.1 Define test-case model and requirement/source trace IDs; verify schema fixtures for positive, negative, boundary, authorization, validation, and regression cases.
- [ ] 1.2 Add read-only `qa plan` input path selection; use filesystem snapshots in tests to prove no project writes, child processes, endpoint requests, or tool installs occur.
- [ ] 1.3 Add explicit-destination plan export; verify default planning writes nothing and invalid destination reports failure without partial-success status.

## 2. Generated script review

- [ ] 2.1 Choose first adapter only after confirming supported framework and tool version; document adapter version and required environment with fixtures.
- [ ] 2.2 Generate draft script and run static syntax/structure validation; verify unsupported adapters are rejected and valid scripts remain `ready-for-review`, never `passed`.
- [ ] 2.3 Add unit/contract test mapping case ID to generated script and expected result; verify generated script has no implicit install or live request.

## 3. Runner policy before platform execution

- [ ] 3.1 Define immutable run manifest with command, cwd, target, redacted env names, mounts, network, output, timeout, and install effects; test manifest hash changes invalidate prior approval.
- [ ] 3.2 Implement fake-runner policy tests for missing consent, stale approval, ambiguous/remote/production DB, no-config disclosure requiring explicit approval, and deceptive `test` name; assert rejected runs spawn zero commands.
- [ ] 3.3 Document actual execution boundary per OS; verify reports do not call host execution or temporary workspace a sandbox and enumerate filesystem/network/endpoint/hardcoded-target limits.
- [ ] 3.4 Implement the first approval-bound runner mode only after its command, target, and DB policy are defined; clear recognized inherited DB host/target variables, block explicit remote/production-like/ambiguous DB settings, warn and require CLI approval when DB configuration is not detected, and test policy with fake runners. Add an OS sandbox adapter only as a separately evidenced capability; never imply it from temp paths.
- [ ] 3.5 Record actual command exit, artifacts, skipped/unavailable/blocked status; tests verify no unavailable, timeout, or blocked case is reported passed.

## 4. Documentation and gates

- [ ] 4.1 Document planning-only default, reviewed consent, DB restrictions, OS capability limits and legacy `verify` effects; verify examples make no unsupported sandbox claim.
- [ ] 4.2 Run unit/contract tests for plan and policy without real target processes; fake runners suffice for local policy contracts. If a separately scoped integration test accesses a DB or claims sandboxing, use a disposable DB and verify the actual OS boundary; obtain separate approval. Do not require Docker, external CI, or human UAT to validate local planning/policy contracts.

## 5. CLI, agent, skills, and optional integrations

- [ ] 5.1 Define CLI-only workflows and agent parity using shared policy and manifests; add testing skill for user projects without implicit install, execution, or network access.
- [ ] 5.2 Define optional local/online AI and MCP configuration; verify planning works without them and preview plus explicit user approval precedes endpoint access and each scoped data transmission, including local integrations.
- [ ] 5.3 Verify install, test, endpoint access, and data transmission each require an explicit preview and user approval bound to exact effects; deny missing, stale, or agent-only approval.

## 6. Phased adapters and OS acceptance

- [ ] 6.1 Define and verify unit/regression pilot adapter and selected framework/version before expanding support.
- [ ] 6.2 Add phased adapter contracts and fixtures for Playwright, Postman collections using Postman CLI, Bruno, and Maestro; keep each unsupported until its own validation and execution boundary are ready.
- [ ] 6.3 Add CLI/agent contract tests for planning, preview, consent, changed scope and results; report untested OS boundaries and human UAT separately without claiming parity.

## 7. Postman setup and integration boundaries

- [ ] 7.1 Pilot Postman CLI first: detect installed version/configuration and preview trusted install source, destination, collection, environment, endpoint, and data before separate setup approval. Missing tool is unavailable; denied approval is blocked; never persist plaintext credentials.
- [ ] 7.2 Define evidence and limits for any future sandbox mode: disposable DB identity, restricted credentials, network policy, fixtures, reset, and cleanup boundary. Do not make these external services prerequisites for local policy tests. Host mode reports actual non-isolated filesystem/process/network/secret exposure and requires exact CLI approval; explicit remote/production-like/ambiguous DB remains blocked, while undetected DB config warns and requires approval.
- [ ] 7.3 Keep M5 provider runtime and service-specific connectors outside current adapter delivery until scope, credential storage, retention, payload consent, idempotency, and permission contracts are approved; basic offline planning stays usable without models.

See `../../CR-ALIGNMENT.md`.

## Workflow follow-up

- The planning-only capability may be applied before a runner implementation is approved.
- Planning and fake-runner policy checks may proceed without external CI, Docker, DB, or human UAT. Host execution requires explicit scope approval, DB preflight, and honest non-isolated reporting; it is not a sandbox fallback. Sandbox-mode execution remains blocked until the requested OS boundary is proven.
- Consume the result schema from `add-scan-history` when linking plan/run records; apply that change first.
