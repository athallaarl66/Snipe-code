# Tasks

## 1. Metric contract and source

- [ ] 1.1 Define versioned metric/config schema with required flags, units, threshold and exclusions; verify schema tests reject missing definitions and round-trip a fixture.
- [ ] 1.2 Implement first deterministic maintainability metric with documented algorithm; run golden fixtures for threshold boundaries and excluded files.
- [ ] 1.3 Implement first duplication metric with documented token/block algorithm; verify duplicate/nonduplicate and exclusion fixtures produce expected counts.
- [ ] 1.4 Add coverage import for one selected report format; verify valid path mapping, malformed/oversized reports, unsupported format and missing report return explicit states.

## 2. Baseline and delta

- [ ] 2.1 Persist baseline reference with result schema, rule/fingerprint versions and base ref; verify reload reconstructs exactly same metadata.
- [ ] 2.2 Compare finding fingerprints for unchanged/new/resolved states; run fixtures for stable findings and verify changed rule version is not silently auto-matched.
- [ ] 2.3 Mark missing base, rename ambiguity and merge conflict unknown; verify no ambiguous finding is reported as resolved.

## 3. Quality gate and docs

- [ ] 3.1 Implement versioned quality gate outcomes `pass`/`fail`/`unknown`/`not-applicable`; test threshold boundaries and prove required missing metric never passes.
- [ ] 3.2 Align CLI exit/output with `harden-verify-safety`; verify mixed unavailable/unknown and explicit fail cases with integration tests.
- [ ] 3.3 Document metric definitions, coverage provenance, baseline matching limits and policy configuration; verify docs distinguish static analysis from test coverage.
- [ ] 3.4 Run focused golden/contract tests plus `cargo test --all-targets --all-features` locally on the pinned toolchain; record rustc version and exit result before marking complete. Actual CI results are recorded separately when available; external CI is not a completion prerequisite.

## CR acceptance clarification

See `../../CR-ALIGNMENT.md`. Independent metrics may proceed after the stable history contract; SAST/SCA-dependent gates wait for their adapters. Missing coverage, baseline, version match, or required metric is unknown/non-pass, never an inferred zero or pass. Keep quality decision values distinct from verify execution states and test the CLI mapping explicitly.

## Workflow follow-up

- Validate the versioned result/fingerprint contract when consumed; independent work does not wait for archive.
- SAST/SCA-dependent gates require the relevant implemented outcome contract.
