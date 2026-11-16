## Context

See proposal.md. The tracked Markdown set contains README.md, docs/README.md, two large feature documents, six scenario folders with four small files each, and three generated-agent source files. The legacy feature documents and scenario snippets predate the current registry and sometimes show IoT templates, unsupported options, or stale function signatures. docs/README.md points to a missing reports directory and an unrelated documentation generator. README.md still calls v0.5.0 the latest empty release, while the public v0.5.3 GitHub release has archives, checksums and signature bundles, and crates.io lists v0.5.3. Release workflow success does not prove installers were exercised on all hosts. The existing `prepare-user-friendly-release` local change has 19 of 21 tasks checked and two publication/evidence tasks pending; it is not a substitute for this follow-up.

## Goals / Non-Goals

Goals: make the published Markdown easy to navigate in English and aligned with current CLI behavior, give QA executable UAT cases, and distinguish verified public facts from unexecuted smoke tests and sign-off. Keep the runtime and existing test contracts stable.

Non-goals: change Rust features or generated stack behavior, fabricate UAT outcomes, publish a new version, modify GitHub settings, claim `v0.5.3` installs successfully on an untested OS, or add UAT bureaucracy to the CLI itself.

## Decisions

### Inventory by tracked files, not by language search alone

Use `git ls-files '*.md'` and examine each tracked file, including generated-agent Markdown. Audit links and code snippets against the CLI parser, registries, tests, and actual sample files. Preserve literal Indonesian program output in code spans when the CLI still emits it, followed by an English explanation. Keep existing folder names, test IDs, and paths to avoid breaking links. Replace prose, rather than mechanically translating unsupported behavior. Alternative: translate strings verbatim; this would preserve false claims about IoT, `--quiet`, and nonexistent Rust types.

### Separate live guidance from historical plans

README.md and docs/features/ describe supported behavior first. Keep `docs/production/` as scenario history only where accurate; replace fictional runnable test snippets with traceable case descriptions or mark them explicitly as unimplemented examples. Do not claim docs/production/*-testing.md is execution evidence. docs/README.md links to actual UAT and report files, marks draft-only artifacts, and drops unsupported `sdd-gen` commands. Alternative: delete all legacy documents; that would lose useful user-story context and break repository links without approval.

### Match release claims to the channel's evidence

Check the latest GitHub release tag, asset names/checksums/bundles, crates.io version, image manifest, tagged CI/release jobs, and documented install paths independently. Record timestamp and exact URLs in a versioned report, not ephemeral pass claims. README states published channels and any untested install paths explicitly. Prefer a tested local smoke installation in a disposable directory for the current host; leave other-host tests unrun until QA executes them. Alternative: infer installer success from a published archive or a green job; neither exercises the user's path.

### Human UAT is distinct from automated evidence

Create an English UAT matrix for the user-facing flows and a v0.5.3 test-report template with separate automated, install-smoke, and human-UAT sections. Each result needs command, environment, expected/actual behavior, outcome, and evidence URL/path. Do not precheck a UAT case or sign for a reviewer. QA owns formal evidence and sign-off. Automated checks can be recorded with the exact CI URL or local command after execution, but cannot impersonate a human acceptance run. A report with pending cases has a pending go/no-go decision.

## Risks / Trade-offs

- Stale release metadata: re-check at edit/verification time and avoid an undated claim about what remains the latest version.
- README tokens are asserted in multiple Rust tests: search assertions before rewriting; change only stale expectations, then run relevant suites.
- Some CLI strings remain Indonesian: English prose can explain their exact current output; translating quoted literals would mislead until CLI behavior changes in a separate change.
- Markdown translation could obscure unsupported historical scenarios: retain IDs, distinguish planned from implemented, and compare each concrete example with registries/code.
- Live installer and verification tests can write files or access the network: use disposable directories; never run `snipe-code verify .` on this repository.
- Current change lives under ignored openspec/: check presence after branch switches and do not imply it ships in the repository.

## Migration Plan

1. Capture tracked Markdown inventory, link map, current public release state, relevant test assertions, and baseline git status.
2. Rewrite README and docs index against verified behavior; revise feature and scenario pages in manageable batches, preserving directory paths and identifying historical content.
3. Add UAT plan and report template; add actual evidence only from commands/CI that were run, keeping every unexecuted case pending.
4. Run prose checks, broken-link/obsolete-claim searches, affected Rust doc-contract tests and local smoke tests in temporary workspaces. Compare final diff and report unresolved QA/host checks.
5. Do not advance the prior change's publication tasks or mark this report approved without the required independent evidence and reviewer decision. If a claim proves false, revert that claim to the last verified state.
