# Tasks

## 1. Rule contract and first supported slice

- [ ] 1.1 Define versioned rule metadata and supported language/construct matrix; verify each released rule has ID, scope, evidence policy, severity, confidence, and matching fixture.
- [ ] 1.2 Add source parsing/dataflow adapter only for selected tested constructs; run positive/negative tests for tainted shell execution, parameterized SQL, escaped HTML, sanitized input, and false-positive safe cases.
- [ ] 1.3 Keep legacy heuristics versioned and separate; verify existing audit fixtures remain stable and rule identity changes do not silently merge fingerprints.
- [ ] 1.4 Add redaction tests using canary credentials; verify canary absent from finding serialization, stdout/stderr, history persistence, and reports.

## 2. Dependency graph and advisory snapshots

- [ ] 2.1 Choose and document first ecosystem input format after evaluating lockfile availability; verify direct/transitive resolution on pinned fixtures and report unsupported/missing inputs as unknown.
- [ ] 2.2 Define snapshot format, provenance, timestamp/freshness, and integrity policy; verify malformed, stale, and absent snapshots yield unknown without network in offline mode.
- [ ] 2.3 Add opt-in snapshot refresh with explicit consent and atomic activation; verify rejection emits zero requests and invalid update preserves prior active snapshot.
- [ ] 2.4 Add affected and unaffected dependency fixtures; verify result lists package/version, advisory ID/source, severity, and snapshot provenance.

## 3. Contract integration and documentation

- [ ] 3.1 Integrate SAST/SCA rule states into versioned result envelope; verify CLI and saved result preserve matching IDs, provenance, unknown, stale, and error states.
- [ ] 3.2 Document only the ecosystems/rules whose fixtures and adapter tests pass; verify unsupported is explicitly distinct from clean.
- [ ] 3.3 Run `cargo test --all-targets --all-features` on pinned CI toolchain plus focused fixture tests; record actual command/exit before marking complete.

## CR acceptance clarification

See `../../CR-ALIGNMENT.md` for dependencies and source requirements. Before closing 2.2, test absent/invalid snapshot timestamps as unknown without egress, not only expired snapshots. Before closing 1.1-1.2, record the selected corpus, supported constructs, and unsupported/timeout behavior; no announced support without paired positive/negative fixtures.

## Workflow follow-up

- Validate compatibility with the available result envelope; independent work does not wait for archive.
- Gates consuming SAST/SCA outcomes require the relevant implemented contract, not archive.
