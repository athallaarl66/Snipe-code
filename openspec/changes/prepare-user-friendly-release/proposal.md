## Why

The CLI builds and its main-branch CI passes, but the published v0.5.0 release has no downloadable assets, the crate is absent from crates.io, and the install-first README presents both paths as ready. A confirmed overwrite can lose an existing project if publication fails; CLI scripting, AI guidance, and the feature documents also promise more than the current behavior reliably delivers. Fix the release path and first-run experience before promoting the project.

## What Changes

- Make overwrite recoverable on failed publication while keeping explicit confirmation and existing symlink safeguards.
- Add validated `new` flags for template, frontend, backend, and database; reject unknown flags, missing values, and conflicting selections before writing. Keep interactive selection for unspecified values and deterministic non-interactive defaults.
- Keep `audit --json` stdout machine-readable even when export format defaults non-interactively; retain the human notice on stderr and report files in the scanned project.
- Make generated AI instructions request human review of heuristic HIGH findings before edits, not automatic blanket fixes. Narrow demonstrably noisy audit patterns with fixture-backed false-positive tests; keep the audit labeled heuristic. Explain opt-in files, slash-command availability, and PATH requirements without claiming a built-in AI service.
- Reconcile the blog preset's static RSS sample and other advertised capabilities with what is actually generated. Do not claim a continuously updating feed unless implementation and tests prove it.
- Replace install-first marketing with an evidence-backed quick start, command decision guide, compact ASCII identity, real CLI demo, honest distribution status and limits. Translate and reconcile both `docs/features/` files to English; retain the tracked Indonesian README unless the owner explicitly decides otherwise. Keep prose plain and mobile-readable.
- Verify release packaging, downloads, signatures, installation, crate and image independently before advertising each channel. Propose discoverability metadata and a real terminal recording after validation; no claim or guarantee of GitHub Trending.

## Capabilities

### New Capabilities

- `ai-agent-guidance`: safe and accurate generated agent rules, skills, and commands for heuristic findings.

### Modified Capabilities

- `safe-project-generation`: preserve the old destination if final overwrite publication fails.
- `cli-process-semantics`: validate `new` arguments and preserve JSON-only stdout for non-interactive audit.
- `generated-project-contracts`: describe the blog RSS sample accurately instead of advertising live updates.
- `product-documentation-accuracy`: document verified installation state, command effects, AI opt-in behavior, and demonstrated limits.
- `release-assurance`: verify actual published channels before marking each installation method available.
- `stack-aware-audit`: reduce confirmed false-positive patterns while preserving reported heuristic scope.

## Impact

`src/main.rs`, `src/commands/new.rs`, `src/generator.rs`, AI content under `templates/ai-tools/`, blog preset/docs, relevant tests, `README.md`, `README.id.md`, `docs/features/`, release workflow/installers only where tests expose a defect, and optional GitHub repository metadata with owner approval. No extra runtime dependency or embedded AI service. Existing local deletion of `README.id.md` is not authorization to remove the translated README; preserve/reconcile it during implementation. The tag/release re-cut, crate publication, image push, and repository-setting changes are separate owner-approved external actions, not automatic side effects of applying this change.
