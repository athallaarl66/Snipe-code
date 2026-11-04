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

## Workflow follow-up

- The planning-only capability may be applied before a runner implementation is approved.
- Runner execution tasks remain blocked until OS threat model and sandbox boundary are reviewed; no host-execution fallback.
- Consume the result schema from `add-scan-history` when linking plan/run records; apply that change first.
