# Tasks

## 1. Verify status and safety

- [x] 1.1 Adapt exit-status tests; unavailable applicable gates never yield complete-pass. Verified `cargo test --test verify_test`.
- [ ] 1.2 Add blocked gate status, aggregate status, text/JSON report coverage for passed, failed, unavailable, blocked, and not-applicable.
- [ ] 1.3 Update CLI help, README, and CLI tests for status/exit codes 0–4.
- [ ] 1.4 Inventory gate effects and preflight; fake-runner tests prove blocked commands do not spawn.
- [ ] 1.5 Classify effective DB settings conservatively, redact credentials, and test precedence, unknown, remote, production-like, and deceptive `test` names.
- [ ] 1.6 Document per-gate filesystem/network effects and preflight limits.
- [ ] 1.7 Require preview and explicit scope-bound user approval for arbitrary CLI targets. Add a narrowly internal `verify-generated` path restricted to its self-created temporary workspace; preserve database preflight. Test no arbitrary path or public bypass.
- [ ] 1.8 Run focused verify unit and CLI suites.

## 2. Audit root safety and scan/export

- [ ] 2.1 Reject symlink/reparse-point entries before dotfile traversal; test POSIX symlinks and Windows fail-closed behavior where supported.
- [ ] 2.2 Verify canonical-root checks prevent outside-root content from entering reports/JSON.
- [ ] 2.3 Document symlink scope and limitations against source/tests.
- [ ] 2.4 Add read-only `scan [PATH] [--json]`; CLI test verifies valid JSON and unchanged project tree.
- [ ] 2.5 Add versioned scan-result export with explicit `--format` and `--output`; validate malformed input before writes.
- [ ] 2.6 Test unwritable output, unknown format, and truthful export errors.
- [ ] 2.7 Document scan/export and legacy behavior.

## 3. Audit CLI compatibility and report

- [ ] 3.1 Strictly reject unknown/duplicate options, duplicate paths, and missing values before target I/O.
- [ ] 3.2 Preserve legacy `audit` output formats, JSON stdout, and exit codes; warn on stderr about report writes.
- [ ] 3.3 Paginate PDF and verify extracted text contains every finding on a large fixture; inspect rendered layout before claiming visual acceptance.
- [ ] 3.4 Correct README and feature docs; verify claims against source/tests.

## 4. Final verification

- [ ] 4.1 Run `cargo test --test verify_test --test verify_cli_test` and audit-focused tests.
- [ ] 4.2 Run full `cargo test --all-targets --all-features`, formatting/lint gates, and build; record exact outcomes.
- [ ] 4.3 Validate OpenSpec change and reconcile checklist only with completed evidence.

## Workflow follow-up

- Archive this unified change after review. Retire the duplicate local `harden-audit-cli` proposal only after its scope and evidence are represented here; do not archive an incomplete change as completed.
