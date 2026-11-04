# Tasks

## 1. Result contract

- [ ] 1.1 Add versioned scan-result envelope and schema fixture; verify JSON golden/contract tests reject unknown major schema versions.
- [ ] 1.2 Add rule status/scope/exclusion and stable versioned finding fingerprint fields; verify repeated fixture runs retain fingerprints while rule-version changes are distinguishable.
- [ ] 1.3 Redact credentials before output/persistence; run canary tests over JSON, logs, DB, and export to verify literal canary never appears.

## 2. Local persistence

- [ ] 2.1 Add opt-in local SQLite store separate from project DB config; verify offline tests open only application data path and never read target DB credentials.
- [ ] 2.2 Add versioned transactional migrations and foreign-key constraints; verify migration success and injected failure rollback preserve prior rows.
- [ ] 2.3 Add history list/show/export operations; verify each uses the canonical result and keeps offline operation network-free.
- [ ] 2.4 Add targeted history deletion and file permission policy; verify deleting one project leaves other run rows and project source unchanged.

## 3. Compatibility and docs

- [ ] 3.1 Keep legacy audit JSON contract readable during migration and document its relationship to the new envelope; verify compatibility fixtures cover prior output.
- [ ] 3.2 Document data location, retention, redaction, and lack of encryption guarantee; verify docs do not claim plaintext SQLite is encrypted.
- [ ] 3.3 Run `cargo test --all-targets --all-features` and migration/schema contract tests on CI toolchain; record toolchain/exit result before marking complete.

## Workflow follow-up

- Apply after `harden-audit-cli` is archived; validate consumer compatibility before archiving this capability.
- Archive before `add-static-dependency-analysis` and `add-quality-gates`, which consume the result schema.
