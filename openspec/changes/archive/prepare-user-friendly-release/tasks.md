## 1. Baseline and safety

- [x] 1.1 Record branch/worktree state, tracked `README.id.md` deletion, current release assets and registry state; read affected tests before edits. Work on a feature branch without restoring or deleting user changes silently.
- [x] 1.2 Add isolated overwrite failure fixtures to `tests/collision_test.rs` that prove old destination survives final publish failure, recovery failure reports the backup path, and successful replacement removes generator-owned temporary data.
- [x] 1.3 Change `src/generator.rs` to swap the old destination to a unique sibling backup after staging validation, roll back a failed publish, and report any retained recovery path. Preserve existing symlink checks and cleanup behavior for a new destination.

## 2. CLI behavior

- [x] 2.1 Add CLI tests for valid fully specified and partial selections, unknown IDs, missing values, unexpected positionals, `--dry-run` alias, and no-stack combination; assert exit 2 and no project mutation for usage failures. Cover `tests/ai_integration_test.rs`, `tests/cli_help_test.rs`, and related generator fixtures pinned to defaults.
- [x] 2.2 Extend `src/main.rs` parsing and `src/commands/new.rs` options to accept `--template`, `--frontend`, `--backend`, and `--database` with registry-backed validation; skip only supplied prompts and preserve unspecified interactive/non-interactive behavior.
- [x] 2.3 Add a structured-output test for `audit PATH --json` without `--export` in a non-interactive shell; fix default notice routing in `src/main.rs` so stdout parses as one JSON object and Markdown is still exported. Keep existing plain `audit` stdout contract in `tests/cli_help_test.rs`.

## 3. Audit and generated content

- [x] 3.1 Add safe/unsafe paired fixtures to existing audit tests for hardcoded-secret and concatenated-query rules, including non-JavaScript supported sources; preserve confirmed HIGH findings with locations while eliminating reproduced benign hits.
- [x] 3.2 Narrow only the reproduced noisy patterns in `src/audit.rs`; retain the heuristic disclosure and existing finding counts/exit semantics for confirmed risks.
- [x] 3.3 Update shared, skill, and command bodies under `templates/ai-tools/content/` to review findings before edits and report false positives. Test opt-in selection, generated paths, and shared content via `tests/ai_integration_test.rs`.
- [x] 3.4 Label the blog RSS URL and entry as a sample in generated guidance and product docs; update `tests/blog_test.rs` to assert the actual static behavior, not an automatically updating feed. Keep `frontend/public/rss.xml` path unless a verified requirement demands change.

## 4. First-run documentation and discoverability

- [x] 4.1 Capture a real terminal transcript from the built CLI in an isolated workspace; choose a short ASCII banner and keep both readable at narrow width. Do not present invented output.
- [x] 4.2 Rewrite `README.md` with verified source-build quick start, current per-channel release status, command decision guide and side effects, AI opt-in guidance, limitations, and a compact real demo. Preserve unrelated literals required by `tests/release_contract_test.rs`, `tests/verify_cli_test.rs`, `tests/ai_integration_test.rs`, `tests/bugfix_test.rs`, and `tests/test_infra_contracts_test.rs`; update tests only where the documented behavior intentionally changes.
- [x] 4.3 README.id.md deletion confirmed intentional by owner; remove obsolete link/test requirement and keep English README as primary. Run doc prose checks for README.md.
- [x] 4.4 Rewrite `docs/features/SNIPE-PRD.md` in English with clear implemented, planned, and limitation sections; remove or mark unsupported flags, IoT examples, live RSS and security claims using code/test references.
- [x] 4.5 Rewrite `docs/features/technical.md` in English against actual modules, embedded templates, release jobs, CLI flags, generated paths and failure cases; remove fictional code snippets and stale JSON/stack examples. Run doc/code contract searches and relevant tests.
- [x] 4.6 Draft GitHub description/topics and optional short terminal recording from a real binary, but do not update repository settings or publish media without owner approval. State that discoverability work cannot guarantee GitHub Trending.

## 5. Release verification and handoff

- [x] 5.1 Run `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test --all-targets --all-features`; resolve regressions including README token contracts without hiding failures.
- [x] 5.2 Run generation and audit smoke tests in temporary directories, then the existing per-stack generated-project and Compose CI matrix in isolated workspaces; do not run `snipe-code verify .` on this repository root. Report unavailable external toolchains explicitly.
- [ ] 5.3 Verify packaging and installer failure paths with staged local archives, then check real public asset count, checksum/signature, installer, crates.io install, and image independently; keep README channels marked unavailable until each actual channel passes.
- [ ] 5.4 Present a release checklist and seek separate owner authorization before any new tag, release re-cut, crate/image publication, or remote repository-setting changes. After authorized publication, read back exact published targets and update only verified availability claims; otherwise leave the task open and report blockers.
- [x] 5.5 Re-run `openspec validate prepare-user-friendly-release --strict`, inspect final worktree/diff and reconcile local-only OpenSpec status; record which checks passed, failed, or remain owner-gated without claiming full release readiness early.
