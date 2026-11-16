# Tasks

## 1. Metric contract and source

- [x] 1.1 Define versioned metric/config schema with required flags, units, threshold and exclusions; verify schema tests reject missing definitions and round-trip a fixture. `cargo test --test quality_test --locked`: schema round-trip and missing/invalid definition cases passed, exit 0.
- [x] 1.2 Implement first deterministic maintainability metric with documented algorithm; run golden fixtures for threshold boundaries and excluded files. `cargo test --test quality_test --locked`: 2 passed, exit 0; inclusive body-line spans, free/impl/trait bodies, exclusions and unsupported-language unknown.
- [x] 1.3 Implement first duplication metric with documented token/block algorithm; verify duplicate/nonduplicate and exclusion fixtures produce expected counts. `cargo test --test quality_test --locked`: 3 passed, exit 0; exact 50-token blocks, comment/whitespace normalization, literal differences and exclusions.
- [x] 1.4 Add coverage import for one selected report format; verify valid path mapping, malformed/oversized reports, unsupported format and missing report return explicit states. `cargo test --test quality_test --locked`: 4 passed, exit 0; LCOV provenance, absolute/relative paths, missing/invalid/unsupported/oversized/duplicate/out-of-range records and exclusions.

## 2. Baseline and delta

- [x] 2.1 Persist baseline reference with result schema, rule/fingerprint versions and base ref; verify reload reconstructs exactly same metadata. `cargo test --test quality_integration_test --locked`: baseline metadata survives SQLite save/read; old v2 envelope without quality remains compatible.
- [x] 2.2 Compare finding fingerprints for unchanged/new/resolved states; run fixtures for stable findings and verify changed rule version is not silently auto-matched. Same command: synthetic evaluated-scope fixtures cover unchanged/new/resolved and rule/fingerprint-version mismatch.
- [x] 2.3 Mark missing base, rename ambiguity and merge conflict unknown; verify no ambiguous finding is reported as resolved. Same command: missing input/ref, staged rename and real unmerged Git index remain unknown with empty new/resolved lists.

## 3. Quality gate and docs

- [x] 3.1 Implement versioned quality gate outcomes `pass`/`fail`/`unknown`/`not-applicable`; test threshold boundaries and prove required missing metric never passes. `cargo test --test quality_integration_test --locked`: 4 passed, exit 0; required missing, mixed unknown/evaluated, explicit fail and disabled config covered.
- [x] 3.2 Align CLI exit/output with `harden-verify-safety`; verify mixed unavailable/unknown and explicit fail cases with integration tests. `cargo test --test quality_test --test quality_integration_test --test verify_test --test verify_cli_test --locked`: 41 passed, 0 failed, exit 0; quality exits 0/1/2/3/4, lowercase text/JSON states, baseline reason, optional missing metric non-pass, usage/runtime/save errors, no implicit history, save/read/export round-trip. Existing verify unavailable/blocked semantics remain unchanged.
- [x] 3.3 Document metric definitions, coverage provenance, baseline matching limits and policy configuration; verify docs distinguish static analysis from test coverage. Techwriter handoff delivered `docs/features/quality-gates.md` and README command link; source reviewed. `cargo test --test quality_integration_test --locked documented_quality_config_runs_with_imported_not_computed_coverage`: 1 passed, exit 0; complete documented config executes through CLI with/without LCOV, preserves provenance and missing input unknown. Prose checker: new doc has zero em dashes/bold/curly quotes/emoji.
- [x] 3.4 Run focused golden/contract tests plus `cargo test --all-targets --all-features` locally on the pinned toolchain; record rustc version and exit result before marking complete. Actual CI results are recorded separately when available; external CI is not a completion prerequisite. `rustc 1.85.0 (4d91de4e4 2025-02-17)`; final sequential `cargo fmt --check`, `cargo clippy --all-targets --all-features --locked -- -D warnings`, `cargo test --all-targets --all-features --locked -j 1`: all exit 0; 245 passed, 0 failed, 0 ignored across 28 result summaries. Initial parallel build failed on missing libsqlite3_sys/hashlink rlib artifacts; serial retry and final gate passed. Full log: `C:/Users/Athalla arli/AppData/Local/hermes/cache/scratch/add-quality-gates-final.log`. Actual CI NOT RUN.

## CR acceptance clarification

See `../../CR-ALIGNMENT.md`. Independent metrics may proceed after the stable history contract; SAST/SCA-dependent gates wait for their adapters. Missing coverage, baseline, version match, or required metric is unknown/non-pass, never an inferred zero or pass. Keep quality decision values distinct from verify execution states and test the CLI mapping explicitly.

## Workflow follow-up

- Validate the versioned result/fingerprint contract when consumed; independent work does not wait for archive.
- SAST/SCA-dependent gates require the relevant implemented outcome contract.
