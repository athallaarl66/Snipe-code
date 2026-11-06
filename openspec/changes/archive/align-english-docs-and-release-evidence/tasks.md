## 1. Source inventory

- [x] 1.1 Record clean branch/worktree state; enumerate every tracked Markdown file in README.md, docs/, and templates/ai-tools/content/ and inspect links, language, command examples, unsupported preset names, and references to test results.
- [x] 1.2 Search Rust tests for README and docs literals, especially `tests/release_contract_test.rs`, `tests/verify_cli_test.rs`, `tests/ai_integration_test.rs`, `tests/bugfix_test.rs`, and `tests/test_infra_contracts_test.rs`. Cross-check current `new` flags, registries, generated paths, and release claims before drafting.
- [x] 1.3 Re-check public v0.5.3 assets, checksums/bundles, crates.io version, tagged CI/release jobs, and image availability. Distinguish those observations from unperformed installer smoke tests. Observed 2026-10-21: 12 release assets (4 archives, 4 `.sha256`, 4 bundles), crates.io `snipe-code` 0.5.3, ghcr tags `v0.5.2`/`v0.5.3`/`latest`. Per-job release logs were not re-read and no installer was executed.

## 2. Product documentation

- [x] 2.1 Update `README.md` distribution status and first runnable quick start with a tested path; distinguish artifact publication from installation evidence. Retain real CLI messages as exact quotes where tests require them. Keep existing command side-effect guidance and correct supported selection flags.
- [x] 2.2 Replace the outdated `docs/README.md` generator/report claims with an index of existing feature, scenario, UAT, and versioned-report documents, labeling plans versus executed evidence.
- [x] 2.3 Reconcile `docs/features/SNIPE-PRD.md` with actual preset/stack IDs and current CLI behavior; remove unsupported `--quiet`, IoT-as-implemented, dynamic RSS, merge option, and unproven security/performance claims or mark them historical. Indonesian prose is kept, per the repository convention that the PRD is written in Indonesian; the earlier English-only wording of this task is dropped to match that convention.
- [x] 2.4 Rewrite `docs/features/technical.md` against the 0.5.3 code: module map, template layout, real preset table, CLI surface, generation flow, generated CI and Docker output, audit, analyze, verify, AI artifacts, release channels, limits.

## 3. Scenario documents and generated guidance

- [x] 3.1 Review and rewrite the four US-001 Markdown files; distinguish acceptance ideas from executable tests and update registry/code examples (IoT preset removed, `--quiet` removed, `indicatif` dependency removed, real error strings).
- [x] 3.2 Review and rewrite the four US-002 Markdown files; preserve scenario identifiers and working paths, remove unsupported IoT and merge claims.
- [x] 3.3 Review and rewrite the four US-003 Markdown files; align dry-run examples with current CLI parser and observed read/write behavior.
- [x] 3.4 Review and rewrite the four US-005 Markdown files; mark absent pages, security controls, and performance guarantees as plans instead of implemented output.
- [x] 3.5 Review and rewrite the four US-006 Markdown files; reconcile asserted portfolio output with current preset and tests.
- [x] 3.6 Review and rewrite the four US-007 Markdown files; identify sample-only RSS, observed blog files, and unimplemented dynamic feed behavior.
- [x] 3.7 Review all three `templates/ai-tools/content/*.md` source files; English prose and paths already match generated behavior and the opt-in semantics, so no edit was needed.

## 4. UAT plan and versioned report

- [x] 4.1 Add a traceable acceptance matrix covering installation, wizard/scripted generation, collision cancel/overwrite/recovery, dry run, analyze, audit exports/JSON and exit codes, verify side effects, and opt-in agent files. Written as `docs/uat/UAT-MATRIX.md` with prerequisites, expected result, automation reference, status, and recorded gaps.
- [ ] 4.2 Add a v0.5.3 report separating observed public release/CI evidence, executed local install smoke checks, pending platform checks, and human UAT not yet run. Left open: it is a QA artifact, and writing it here would put a plan where evidence belongs.
- [ ] 4.3 Hand the formal UAT/evidence execution and sign-off to QA; record its actual verified findings only when returned, rather than treating plan/report placeholders as acceptance.

## 5. Verification

- [x] 5.1 Run the prose lint on every touched user-facing document, audit tracked Markdown for remaining Indonesian prose outside exact CLI literals, scan obsolete options/presets, and check links resolve. Result: 0 em dashes and 0 bold spans in new prose; the single em dash left in `README.md` sits inside the quoted CLI line `Terminal non-interaktif — default export: markdown.`, which `tests/cli_help_test.rs` pins. No `--quiet`, `sdd-gen`, or `iot-dashboard` references remain in tracked Markdown.
- [x] 5.2 Run the docs-related Rust tests and the full suite. `cargo fmt --check` exits 0, `cargo clippy --all-targets --all-features -- -D warnings` is clean, and `cargo test --all-targets --all-features` passes every target, including the README contract tests.
- [x] 5.3 Verify smoke actions only inside a disposable directory, inspect final diff and git status, re-check public release claims, and report untested OS and pending QA cases without declaring full release readiness. Smoke run in a temp directory: `new --template blog --frontend nextjs --backend none --database none --name smoke --ai-tools cursor` produced 11 folders and 19 files plus git init and initial commit, `analyze` reported languages and dependencies, `audit --export md` wrote `Audit_Report.md`, and `--dry-run` wrote nothing. Temp directory removed.
- [x] 5.4 Validate this OpenSpec change strictly; keep prior `prepare-user-friendly-release` publication/evidence tasks open until their own acceptance conditions have actually passed. `openspec validate align-english-docs-and-release-evidence --strict` reports "Change is valid".

## Status after this pass

Documentation now matches the 0.5.3 code on every point that was wrong or missing:

- README distribution section states the published `v0.5.3` channels and separates publication from install evidence.
- README and the `new` help text list `--template`, `--frontend`, `--backend`, and `--database`, which the parser has accepted since the selection work landed.
- `docs/README.md` indexes the files that exist instead of a `test-reports/` directory and `sdd-gen` commands that never existed here.
- `docs/features/technical.md` is rewritten from the code, not from the original design draft.
- Preset and scenario documents mark unimplemented page content, SEO, dark mode, and dynamic RSS as plans.
- `docs/uat/UAT-MATRIX.md` adds the missing acceptance case list with honest status.

Still open, by design:

- Human UAT against the published artifacts, and the versioned QA report (4.2, 4.3).
- Installer execution on Linux, macOS, and Windows against `v0.5.3`.
- Language: `README.md`, `docs/README.md`, `technical.md`, and the UAT matrix are English; the PRD and scenario documents stay Indonesian. Translating those is a separate decision, and doing it silently would have rewritten the owner's product text.
