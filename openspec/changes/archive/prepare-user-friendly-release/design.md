## Context

See proposal.md. The code uses a manual parser in `src/main.rs`, non-interactive defaults in `src/commands/new.rs`, and staging followed by destination deletion and rename in `src/generator.rs`. The AI artifacts share content under `templates/ai-tools/content/`; the blog RSS file is a static sample. `README.id.md` is tracked but currently deleted locally. The release job for `v0.5.0` failed; the published release has no assets and the crate is not on crates.io. OpenSpec and ROADMAP are local-only and gitignored.

## Goals / Non-Goals

Goals: maintain a single Rust executable and offline-first analysis, protect prior user data on failed overwrite, provide scriptable generation, keep stdout JSON stable, and distinguish tested release channels from intentions. Use existing dependencies and test patterns.

Non-goals: bundle an AI service, automatically fix security findings, implement a full SAST engine, add AI vendors solely for marketing, guarantee GitHub Trending, implement a dynamic RSS publishing system, or silently modify existing tags/releases or remote repository settings.

## Decisions

### Overwrite as a recoverable swap

Stage and validate first as today. For an approved overwrite, rename the existing destination to a unique sibling backup, then rename staging into the destination. If publication fails, rename backup back; if restoration fails, retain backup and print its absolute recovery path. After successful publication, remove backup; if cleanup fails, report its path and avoid claiming a clean success. Verify symlink/junction protections before moving. Prefer sibling renames on the same volume; avoid deleting the old directory first. Alternative: copy over files or delete then rename; both allow partial or unrecoverable loss. Test failure injection through a narrow filesystem boundary or a reproducible controlled failure, never by harming a real directory.

### Explicit flags, defaults only for omitted fields

Extend the existing manual parser; do not add clap. Accept `--template`, `--frontend`, `--backend`, `--database`, `--name`, and `--ai-tools`; validate IDs using the embedded template/stack registries and the existing selection validator. An explicit flag skips its prompt, while missing fields prompt interactively or take the existing non-interactive defaults. Reject unknown flags, positional arguments, missing values, and invalid combinations before collision or filesystem operations. Keep the `--dry-run` alias by parsing its first token as an option instead of ignoring it. Alternative: a second non-interactive command or a general parser dependency; neither is needed.

### Structured audit output and AI guidance

Move the default-format notice to stderr only in JSON mode; preserve the plain text command's stdout behavior. Do not change report location or policy exit codes. Start audit precision work with minimal safe/unsafe fixture pairs for hardcoded secret and concatenated SQL patterns in supported extensions; only narrow a rule once both are reproduced, and leave other rules untouched. Replace blanket `fix every HIGH` instructions in shared, skill, and command bodies with source review, false-positive reporting, confirmed-fix verification, and repeat audit. Keep `.ai-workspace` always generated and tool-specific artifacts opt-in. The slash command is an external agent artifact and requires `snipe-code` on PATH; the CLI contains no model integration.

### RSS and documentation truth

Keep the sample RSS in the blog preset, mark its localhost URL and single generated entry as examples requiring manual replacement, and remove claims that adding posts updates it. This is smaller and more honest than introducing a runtime feed generator. Read `README.id.md` from HEAD before touching its local deletion; preserve it as the linked Indonesian translation when applying unless owner explicitly changes that decision. Rewrite both `docs/features/` files in English as present-state documents: implemented behavior first, plans and limitations clearly separated. Preserve tested README literals unless tests change for a justified behavior update. Use a short ASCII banner and a terminal transcript captured from the actual binary, not invented output. No new marketing runtime dependencies.

### Release-channel gating

Keep local quality checks distinct from actual publication: clean-checkout CI and per-stack generated verification; tag-matched archive matrix; archive contents, checksum, and cosign bundle; installer smoke on published assets; successful registry install; separately running image. Advertise each channel only after its independent evidence exists. Do not amend/reuse `v0.5.0` without approval; prefer a new semver tag after a verified candidate if the owner agrees. GitHub description/topics and a real terminal recording are adoption steps, not release-quality evidence. Install docs must stay truthful while a channel is missing.

## Risks / Trade-offs

- Rename rollback can fail because of permissions or concurrent edits: retain the backup with its exact path; do not erase it during cleanup.
- Existing tests pin README text and blog file paths: inspect and adjust only affected assertions, retaining unrelated contracts.
- Working tree already deletes `README.id.md`: treat as unresolved user state, not permission to erase the translation.
- New flags allow invalid cross-stack choices: validate before staging and assert that no project appears on usage error.
- Remote release and package registry need credentials/owner action: report blockers without faking assets or publishing from an unverified tree.
- Static RSS cannot stay synchronized with new posts: label it as a sample; add a feed implementation in a later change only if users need live updates.

## Migration Plan

1. Capture existing tests and worktree state; implement the safety and CLI changes with isolated fixtures; then update AI/blog guidance.
2. Update README, README.id.md, and both English feature documents from observed CLI behavior. Verify links and prose as well as literal test contracts.
3. Run local Rust gates and generated-stack verification in disposable workspaces, never `verify` on this repository root. Run installer tests against a staged archive, then independently check real public channels.
4. Until a new release is published and tested, show source build as the working quick start and mark broken distribution paths unavailable. Only after explicit owner approval, create/publish a new release; verify assets, checksum/signature, installs, registry, and image before updating channel claims.
5. If rollout fails, leave public documentation at the last verified availability state; preserve the old tag/release for diagnosis. Do not delete public tags or releases without separate approval.

## Open Questions

None block implementation. The owner can decide later whether and when to publish a new version or apply optional repository description/topics and a terminal recording.
