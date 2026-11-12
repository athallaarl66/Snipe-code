# Tasks

## 1. Result contract

- [x] 1.1 Add versioned scan-result envelope and schema fixture; verify JSON golden/contract tests reject unknown major schema versions.
- [ ] 1.2 Add rule status/scope/exclusion and stable versioned finding fingerprint fields; verify repeated fixture runs retain fingerprints while rule-version changes are distinguishable.
- [ ] 1.3 Redact credentials before output/persistence; run canary tests over JSON, logs, DB, and export to verify literal canary never appears.

## 2. Local persistence

- [ ] 2.1 Add opt-in local SQLite store separate from project DB config; verify offline tests open only application data path and never read target DB credentials.
- [x] 2.2 Add versioned transactional migrations and foreign-key constraints; verify migration success and injected failure rollback preserve prior rows.
- [ ] 2.3 Add history list/show/export operations; verify each uses the canonical result and keeps offline operation network-free.
- [ ] 2.4 Add targeted history deletion and file permission policy; verify deleting one project leaves other run rows and project source unchanged.

## 3. Compatibility and docs

- [x] 3.1 Keep legacy audit JSON contract readable during migration and document its relationship to the new envelope; verify compatibility fixtures cover prior output.
- [ ] 3.2 Document data location, retention, redaction, and lack of encryption guarantee; verify docs do not claim plaintext SQLite is encrypted.
- [x] 3.3 Run `cargo test --all-targets --all-features` and migration/schema contract tests on CI toolchain; record toolchain/exit result before marking complete.
  Local Windows: rustc 1.85.0 (matching CI pin), `cargo test --all-targets --all-features` exit 0; history unit tests exit 0. CI runner not executed.

## 4. CR result and deletion integrity

- [ ] 4.1 Report executed rule groups as evaluated and unsupported groups as unknown; test scan, save/load, to_audit_result, and export round trips preserve exact group states.
- [ ] 4.2 Recompute severity counts, highest_severity, and policy_failed from findings at every untrusted envelope consumer; reject tampered summaries before save/show/export writes.
- [ ] 4.3 Add history delete preview with exact project identity and run count, dry-run without deletion, and explicit confirmation before targeted deletion. Test refusal, stale preview, non-interactive missing approval, unrelated rows, and unchanged source.
- [ ] 4.4 Verify actual OS privacy behavior: Unix permissions and Windows inherited ACL limitation or tested hardening. Do not claim encryption or Windows private ACLs from Unix mode tests.
- [ ] 4.5 Reconcile existing task 3.3 evidence: local pinned-toolchain pass is not CI-runner execution. Capture actual CI evidence or leave CI acceptance explicitly open.

See `../../CR-ALIGNMENT.md`; new tasks remain unchecked until execution evidence exists.

## Workflow follow-up

- Apply after `harden-verify-safety` is archived; validate consumer compatibility before archiving this capability.
- Archive before `add-static-dependency-analysis` and `add-quality-gates`, which consume the result schema.
