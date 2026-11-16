# Tasks

## 1. Rule contract and first supported slice

- [x] 1.1 Define versioned rule metadata and supported language/construct matrix; verify each released rule has ID, scope, evidence policy, severity, confidence, and matching fixture.
- [x] 1.2 Add source parsing/dataflow adapter only for selected tested constructs; run positive/negative tests for tainted shell execution, parameterized SQL, escaped HTML, sanitized input, and false-positive safe cases.
- [x] 1.3 Keep legacy heuristics versioned and separate; verify existing audit fixtures remain stable and rule identity changes do not silently merge fingerprints.
- [x] 1.4 Add redaction tests using canary credentials; verify canary absent from finding serialization, stdout/stderr, history persistence, and reports.

## 2. Dependency graph and advisory snapshots

- [x] 2.1 Choose and document first ecosystem input format after evaluating lockfile availability; verify direct/transitive resolution on pinned fixtures and report unsupported/missing inputs as unknown.
- [x] 2.2 Define snapshot format, provenance, timestamp/freshness, and integrity policy; verify malformed, stale, and absent snapshots yield unknown without network in offline mode.
- [x] 2.3 Add opt-in snapshot refresh with explicit consent and atomic activation; verify rejection emits zero requests and invalid update preserves prior active snapshot.
- [x] 2.4 Add affected and unaffected dependency fixtures; verify result lists package/version, advisory ID/source, severity, and snapshot provenance.

## 3. Contract integration and documentation

- [x] 3.1 Integrate SAST/SCA rule states into versioned result envelope; verify CLI and saved result preserve matching IDs, provenance, unknown, stale, and error states.
- [x] 3.2 Document only the ecosystems/rules whose fixtures and adapter tests pass; verify unsupported is explicitly distinct from clean.
- [x] 3.3 Run `cargo test --all-targets --all-features` locally on the pinned toolchain plus focused fixture tests; record actual command/exit before marking complete. Actual CI results are recorded separately when available; external CI is not a completion prerequisite.

## CR acceptance clarification

See `../../CR-ALIGNMENT.md` for dependencies and source requirements. Before closing 2.2, test absent/invalid snapshot timestamps as unknown without egress, not only expired snapshots. Before closing 1.1-1.2, record the selected corpus, supported constructs, and unsupported/timeout behavior; no announced support without paired positive/negative fixtures.

## Local evidence

- 1.1–1.2: `cargo test --test sast_test --locked`, rustc 1.85.0, exit 0; five tests cover paired fixtures, HTML escaping context, literal braces, unsupported transforms/control flow, input budgets and parse error redaction. Corpus and exclusions recorded in design and embedded rule catalog.

- 1.3–1.4: `cargo test --test sast_test --test static_integration_test --test bugfix_test --test structured_output_test --locked`, exit 0; 43 tests passed. Versioned SAST fingerprint differs from legacy; CLI scan/save/history show/export and MD/PDF/DOCX output plus SQLite bytes exclude canary. Legacy audit and old v2 envelope fixtures pass.

## Workflow follow-up

- Final focused gate: `cargo test --test sca_test --test sast_test --test static_integration_test --locked`, exit 0; 16 passed (8 SCA, 6 SAST, 2 integration). Tests exercise direct/transitive pinned graph, affected/unaffected synthetic OSV pairs, missing/invalid/future timestamps, malformed/hash-corrupt snapshots, stale boundary, zero-fetch/no-write refusal and atomic pointer replacement preserving old data on invalid refresh. CLI scan with missing and fresh/stale snapshots and history readback retain IDs/provenance/status. Invalid imported package status is rejected; source analysis errors survive canonical serialization.
- Final full gate: `cargo test --all-targets --all-features --locked`, rustc 1.85.0, exit 0; 233 passed, 0 failed, 0 ignored across 26 result summaries. `cargo fmt --check`, `cargo clippy --all-targets --all-features --locked -- -D warnings` and `git diff --check` also passed. No live OSV refresh, container, deployment or CI run performed. README records tested Rust/npm slices and offline/unknown/stale limits; prose checker flags existing em dash and `machine-readable`, neither introduced by this change.

- Validate compatibility with the available result envelope; independent work does not wait for archive.
- Gates consuming SAST/SCA outcomes require the relevant implemented contract, not archive.
