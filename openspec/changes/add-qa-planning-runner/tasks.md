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
- [ ] 3.2 Implement fake-runner policy tests for missing consent, stale approval, unknown/remote/production DB and deceptive `test` name; assert rejected runs spawn zero commands.
- [ ] 3.3 Evaluate process/filesystem/network isolation per OS and document capability matrix; verify unsupported boundary returns `blocked` and does not fall back to host execution.
- [ ] 3.4 Implement first sandbox adapter only after OS boundary acceptance; test disposable DB isolation, secret redaction, timeout/cancel and cleanup confined to run workspace.
- [ ] 3.5 Record actual command exit, artifacts, skipped/unavailable/blocked status; tests verify no unavailable, timeout, or blocked case is reported passed.

## 4. Documentation and gates

- [ ] 4.1 Document planning-only default, reviewed consent, DB restrictions, OS capability limits and legacy `verify` effects; verify examples make no unsupported sandbox claim.
- [ ] 4.2 Run unit/contract tests for plan and policy without real target processes; execute sandbox integration only on disposable DB and isolated host after environment review, recording actual result.

## 5. CLI, agent, skills, and optional integrations

- [ ] 5.1 Define CLI-only workflows and agent parity using shared policy and manifests; add testing skill for user projects without implicit install, execution, or network access.
- [ ] 5.2 Define optional local/online AI and MCP configuration; verify planning works without them and preview plus explicit user approval precedes endpoint access and each scoped data transmission, including local integrations.
- [ ] 5.3 Verify install, test, endpoint access, and data transmission each require an explicit preview and user approval bound to exact effects; deny missing, stale, or agent-only approval.

## 6. Phased adapters and OS acceptance

- [ ] 6.1 Define and verify unit/regression pilot adapter and selected framework/version before expanding support.
- [ ] 6.2 Add phased adapter contracts and fixtures for Playwright, Postman collections using Postman CLI, Bruno, and Maestro; keep each unsupported until its own validation and execution boundary are ready.
- [ ] 6.3 Perform and record CLI/agent parity UAT per supported OS: planning, preview, approval and refusal, changed scope, results, and unsupported capabilities; retain blocked status for unverified OS boundaries.

## 7. Postman setup and integration boundaries

- [ ] 7.1 Pilot Postman CLI first: detect installed version/configuration and preview trusted install source, destination, collection, environment, endpoint, and data before separate setup approval. Missing tool is unavailable; denied approval is blocked; never persist plaintext credentials.
- [ ] 7.2 Record disposable DB unique identity, restricted credentials, network deny-by-default, fixtures, reset, and cleanup boundary. Test non-DB runs also enforce filesystem/process/network/secret isolation; lack of evidence blocks execution.
- [ ] 7.3 Keep M5 provider runtime and service-specific connectors outside current adapter delivery until scope, credential storage, retention, payload consent, idempotency, and permission contracts are approved; basic offline planning stays usable without models.

See `../../CR-ALIGNMENT.md`.

## Workflow follow-up

- The planning-only capability may be applied before a runner implementation is approved.
- Runner execution tasks remain blocked until OS threat model and sandbox boundary are reviewed; no host-execution fallback.
- Consume the result schema from `add-scan-history` when linking plan/run records; apply that change first.
