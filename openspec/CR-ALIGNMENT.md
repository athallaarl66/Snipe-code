# CR alignment and implementation route

Source: `../docs/features/cr.md`, sections 1-9. That document remains a proposed product direction and historical audit snapshot, not proof of delivery or approval. This alignment changes planning artifacts only; it does not implement features or complete acceptance tests.

## Read and delivery order

Read this route, then the selected change's proposal, design, delta specs, and tasks. Durable `specs/` and archived changes remain unchanged until the normal reviewed archive process.

1. `harden-verify-safety`: M0 safety, compatibility, generated-project regressions, immediate agent consent wording.
2. `add-scan-history`: M1 result integrity and local history; validate M0 consumer compatibility without waiting for archive.
3. `add-static-dependency-analysis`: M2/M3 supported SAST slices and offline SCA after the stable result contract.
4. `add-quality-gates`: M2 metrics, coverage import, baseline/delta. SAST/SCA-dependent gates wait for change 3; independent metric work need not wait for unrelated adapters.
5. `add-qa-planning-runner`: M4 planning may proceed after M1; execution waits for reviewed OS boundaries and proven isolation. Postman CLI is the first adapter pilot.
6. `clarify-cli-and-agent-workflows`: shared safety and runner contracts govern workflow execution. Independent launcher/copy/report work may proceed; no future command is advertised as available.

Independent work does not wait for archive; verify each consumed contract. No sandbox, Docker, target tests, DB access, installation, or online refresh is authorized by this planning update.

## Traceability

Paths below are relative to `changes/`; task IDs belong to the named change. Requirement labels identify headings in the linked delta, not claims of current implementation.

| CR area | Requirement location | Task owner and IDs | Acceptance method | Dependency |
| --- | --- | --- | --- | --- |
| M0 status, effects, approval, private harness | `harden-verify-safety/specs/safe-verification/spec.md` | harden-verify-safety 1.1-1.8 | Fake runner proves rejection before spawn; CLI text/JSON and exits 0-4; no public auto-approval API | Existing verify consumers |
| Database target preflight | `harden-verify-safety/specs/safe-verification/spec.md`: Database bersama tidak disentuh tanpa verifikasi target dan izin | harden-verify-safety 1.5 | Missing/ambiguous, remote, production-like, deceptive names, precedence and redaction negatives; classification is not proof of isolation | Existing preflight remains; no separate proof-of-no-DB feature |
| Scan/export, legacy parser, symlink boundary | `harden-verify-safety/specs/audit-cli/spec.md` | harden-verify-safety 2.1-2.7, 3.1-3.2, 5.7 | Unchanged source tree; invalid input before I/O; outside-root canary absent; POSIX symlink, Windows junction/file-symlink evidence separate | M0 |
| Override traversal | `harden-verify-safety/specs/generation-safety-regressions/spec.md`: Template override paths stay inside their boundary | harden-verify-safety 5.1 | Malicious folder/file/template-source fixture, drive/UNC and link escape; unchanged outside canary/destination | Before scaffolding safety closure |
| PDF bounds, wrapping, footer, report parity | `harden-verify-safety/specs/audit-cli/spec.md`: Export preserves content inside page boundaries | harden-verify-safety 3.3, 5.3-5.4 | Zero/one/many finding coordinate assertions; extracted identities/counts across JSON/MD/DOCX/PDF; report rendered review separately when available | Missing extractor evidence stays NOT RUN |
| Compose and generated Go CI | `harden-verify-safety/specs/generation-safety-regressions/spec.md` | harden-verify-safety 5.5-5.6 | Config identity assertions, approved disposable healthcheck/DB connection, generated Go workflow without go.sum | Runtime/CI verification optional; separate Docker consent |
| M1 evaluated coverage, summary integrity | `add-scan-history/specs/scan-history/spec.md`: Envelope summaries match findings | add-scan-history 1.2, 4.1-4.2 | Golden round trip; tampered counts/highest severity/policy rejected at each input boundary | Stable M0 envelope consumers |
| Redaction, opt-in local data, migration/privacy | same: Data sensitif tidak disimpan plaintext; Migrasi dan penghapusan riwayat aman | add-scan-history 1.3, 2.1-2.4, 3.2, 4.4-4.5 | Literal canary through stdout/stderr/JSON/SQLite/reports; failure-injection rollback; per-OS permissions; local versus CI evidence separate | Unique product-data fixture, never target DB |
| History delete preview and consent | same: Deletion requires preview and explicit confirmation | add-scan-history 4.3 | Exact project/count, dry-run, declined/missing/stale approval, unchanged unrelated rows/source | M1 |
| M2/M3 SAST scope and candidates | `add-static-dependency-analysis/specs/static-analysis/spec.md` | add-static-dependency-analysis 1.1-1.4, 3.1-3.3 | Positive/negative per announced construct, sanitizer/parameterization, unsupported/error/timeout unknown; versioned legacy rules | M0 then M1; scope/corpus decision before support claims |
| Offline SCA and opt-in snapshot refresh | `add-static-dependency-analysis/specs/dependency-analysis/spec.md` | add-static-dependency-analysis 2.1-2.4 | Direct/transitive pinned fixtures; missing timestamp/stale/invalid snapshot unknown; denied refresh zero requests | Advisory source/license/freshness decision |
| M2 quality, coverage provenance, baseline/PR | `add-quality-gates/specs/quality-gates/spec.md` | add-quality-gates 1.1-1.4, 2.1-2.3, 3.1-3.4 | Metric boundaries, missing coverage not computed, missing base/rename/conflict/version mismatch unknown, required missing metric non-pass | M1; SCA/SAST gates also change 3 |
| M4 reviewed cases/scripts, read-only plan | `add-qa-planning-runner/specs/qa-planning/spec.md` | add-qa-planning-runner 1.1-1.3, 2.1-2.3 | Requirement-case-script IDs; filesystem/process/network no-action checks; validated script is reviewable, not passed | M1; no target execution |
| Postman CLI setup | `add-qa-planning-runner/specs/qa-runner/spec.md`: Postman setup is separately reviewed | add-qa-planning-runner 7.1, 6.2 | Version/source/destination/collection/environment/endpoint/data preview; denied setup blocked, missing tool unavailable | Postman pilot before later adapters |
| M4 isolated execution and parity | same: Eksekusi fail closed bila isolasi tidak tersedia; UAT parity CLI dan agent per OS | add-qa-planning-runner 3.1-3.5, 4.1-4.2, 5.1-5.3, 6.1-6.3, 7.2 | Immutable approval; disposable unique DB/restricted credentials; denied network/secrets; non-DB isolation; timeout/cancel/cleanup; automated contract tests; human UAT optional | Reviewed boundary; unsupported execution is blocked |
| Immediate agent consent fix | `clarify-cli-and-agent-workflows/specs/ai-agent-workflows/spec.md`: Workflow instructions do not grant implicit consent | harden-verify-safety 5.8; clarify-cli-and-agent-workflows 7.2 | Every generated host variant requires scoped preview/approval; no edit/test/network from mere invocation | Immediate wording fix M0; full registry redesign later |
| CLI first-run clarity, English wizard, ownership | clarify-cli-and-agent-workflows `specs/interactive-command-launcher/spec.md`, `specs/safe-project-generation/spec.md`, `specs/product-documentation-accuracy/spec.md` | clarify-cli-and-agent-workflows 1.1-2.8, 4.1-5.8, 7.1, 7.4 | No action before choice; preview/back/cancel/Git separately; distinguish scaffold, execution, reports, agent authoring | Existing scan/export/history explained; future commands unavailable |
| Deterministic verify reports and non-pass | `clarify-cli-and-agent-workflows/specs/verification-report-export/spec.md`: Incomplete results remain non-pass after export | clarify-cli-and-agent-workflows 3.1-3.7, 7.3 | Same result JSON/JUnit/MD; exact states; successful export preserves 0/1/3/4, usage 2, write failure 3 | M0 status/approval; fake runner tests |
| M5 optional model/provider/service connectors | QA planning optional-integration requirements only; full runtime not delivered by these six changes | add-qa-planning-runner 5.2, 7.3 | Basic offline path without model; any approved later scope needs payload redaction/consent, OS credential references, retention, idempotency, permission/timeout/duplicate tests | Deferred scope decision; no silent runtime implementation |

## Status and decisions

Existing checked tasks are preserved, not re-certified by this update. New tasks remain unchecked. In particular, add-scan-history 3.3 records a local run on the pinned toolchain, not a CI run; report actual CI status separately. Spec validation proves artifact structure, not implementation, test coverage, visual PDF quality, sandbox capability, or release readiness.

M5 online AI and Figma/Figcraft/Taiga/Jira/Slack adapters remain proposed future scope. Generic optional-integration contracts do not deliver those services. Approve provider/service scope and credential, retention, permission, and transport contracts before a separate runtime implementation. No new change is created here.

Unresolved implementation choices remain explicit: first supported language/construct corpus, advisory source/freshness/license, metric/coverage format, and OS sandbox capability. Do not invent support to close a checkbox. Compatibility and safety work can proceed independently where those choices are not needed.

All product surfaces should disclose target, effects, current support, and recovery action. Legacy audit writes security reports; scan is read-only unless explicit save is requested; verify executes declared commands and is not a sandbox; generated instructions are not runtime permissions. Full-product menus must not claim future QA/gate/connector commands exist.
