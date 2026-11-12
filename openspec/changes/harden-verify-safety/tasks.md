# Tasks

## 1. Verify status and safety

- [x] 1.1 Adapt exit-status tests; unavailable applicable gates never yield complete-pass. Verified `cargo test --test verify_test`.
- [x] 1.2 Add blocked gate status, aggregate status, text/JSON report coverage for passed, failed, unavailable, blocked, and not-applicable. Verified `cargo test --test verify_test report_tracks_all_gate_states_and_serializes_aggregate_status`.
- [x] 1.3 Update CLI help, README, and CLI tests for status/exit codes 0–4. Verified `cargo test --test verify_test --test verify_cli_test` (30 passed).
- [x] 1.4 Inventory gate effects and preflight; fake-runner tests prove blocked commands do not spawn. `cargo test --test verify_test --test verify_cli_test`: 30 passed, exit 0; DB classification limitations remain under 1.5.
- [ ] 1.5 Classify effective DB settings conservatively, redact credentials, and test precedence, unknown, remote, production-like, and deceptive `test` names. Partial: SQLite remote/production URL, URL override, unsupported scheme and conflicting SQLite settings tested with `cargo test --lib database_safety_tests`; effective config precedence and redaction remain unverified; target classification is not proof of isolation.
- [ ] 1.6 Document per-gate filesystem/network effects and preflight limits.
- [ ] 1.7 Require preview and explicit scope-bound user approval for arbitrary CLI targets. Add a narrowly internal `verify-generated` path restricted to its self-created temporary workspace; preserve database preflight. Test no arbitrary path or public bypass.
- [x] 1.8 Run focused verify unit and CLI suites. `cargo test --test verify_test --test verify_cli_test`: 23 + 7 passed, exit 0.

## 2. Audit root safety and scan/export

- [ ] 2.1 Reject symlink/reparse-point entries before dotfile traversal; test POSIX symlinks and Windows fail-closed behavior where supported.
- [ ] 2.2 Verify canonical-root checks prevent outside-root content from entering reports/JSON.
- [ ] 2.3 Document symlink scope and limitations against source/tests.
- [ ] 2.4 Add read-only `scan [PATH] [--json]`; CLI test verifies valid JSON and unchanged project tree.
- [ ] 2.5 Add versioned scan-result export with explicit `--format` and `--output`; validate malformed input before writes.
- [ ] 2.6 Test unwritable output, unknown format, and truthful export errors.
- [ ] 2.7 Document scan/export and legacy behavior.

## 3. Audit CLI compatibility and report

- [x] 3.1 Strictly reject unknown/duplicate options, duplicate paths, and missing values before target I/O. `cargo test --test structured_output_test audit_rejects_ambiguous_options_before_target_io` passed; CLI dispatch validates before target access.
- [ ] 3.2 Preserve legacy `audit` output formats, JSON stdout, and exit codes; warn on stderr about report writes.
- [ ] 3.3 Paginate PDF and verify extracted text contains every finding on a large fixture; report visual inspection separately if not performed.
- [ ] 3.4 Correct README and feature docs; verify claims against source/tests.

## 5. CR safety and report acceptance

- [ ] 5.1 Validate override folder paths, file paths, and template sources before any staging read/write; reject traversal, absolute/drive/UNC paths, and symlink/reparse escapes. Test outside-root canaries and unchanged destination on rejection.

- [ ] 5.3 Restrict PDF coordinates to A4 margins for zero, one, and many findings; wrap long text, preserve full finding content, and add footer per page. Verify every finding by text extraction; report visual inspection separately when available.
- [ ] 5.4 Compare canonical finding identities/counts across JSON, Markdown, DOCX, and PDF exports; test long paths/descriptions and report missing extractor as NOT RUN, not PASS.
- [ ] 5.5 Align generated Compose DB name/user/password with backend configuration, remove obsolete version field, and add healthcheck and configuration tests. Runtime connection verification is optional and requires separate Docker approval; do not claim runtime proof if not run.
- [ ] 5.6 Handle generated Go CI without go.sum using valid cache configuration or generated lock data; verify generated workflow configuration locally; report actual CI status separately when available.
- [ ] 5.7 Extend outside-root canary coverage to Windows file symlinks where supported; report missing symlink privilege as SKIPPED with junction coverage separate.
- [x] 5.8 Patch existing agent command guidance immediately to require separate preview/approval before export writes, source fixes, tests, install, endpoints, or data transmission; do not wait for full workflow redesign. Verify all registry-generated variants. `cargo test --test ai_integration_test`: 17 passed, including registry command variants. Guidance is advisory, not runtime enforcement.

See `../../CR-ALIGNMENT.md` for scope and delivery order.

## 4. Final verification

- [ ] 4.1 Run `cargo test --test verify_test --test verify_cli_test` and audit-focused tests.
- [ ] 4.2 Run full `cargo test --all-targets --all-features`, formatting/lint gates, and build; record exact outcomes.
- [ ] 4.3 Validate OpenSpec change and reconcile checklist only with completed evidence.

## Workflow follow-up

- Archive this unified change after review. Retire the duplicate local `harden-audit-cli` proposal only after its scope and evidence are represented here; do not archive an incomplete change as completed.
