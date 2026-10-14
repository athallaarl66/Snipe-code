# Roadmap

The planned work for this repository lives in `openspec/changes/` as five
OpenSpec changes. This document says what each one does, what order to work
through them, and why that order is constrained.

Current version: `0.1.0`. **Nothing here is implemented yet** — all five changes
are planning artifacts (proposal, design, specs, tasks).

---

## The five changes

| # | Change | Tasks | Capability files created | Capability files extended |
|---|--------|-------|--------------------------|---------------------------|
| 1 | `fix-ci-and-report-truth` | 15 | `cli-interface`, `project-scaffolding`, `security-audit`, `ai-workspace-generation` | — |
| 2 | `refactor-templates-and-test-scaffold` | 23 | `template-sources` | `project-scaffolding` |
| 3 | `feat-verify` | 19 | `project-verification` | — |
| 4 | `feat-ai-integration` | 19 | `ai-tool-integration` | `ai-workspace-generation` |
| 5 | `feat-analyze-and-ship` | 33 | `project-analysis`, `distribution` | `security-audit` |

109 tasks, 27 planning artifacts, 12 spec deltas covering 9 capabilities.

---

## Order of work

```
  STEP 1  fix-ci-and-report-truth                    (CI red -> green; unblocks all)
            |
            +---> STEP 2  refactor-templates-and-test-scaffold
            |               (shares project-scaffolding)
            |
            +---> STEP 4  feat-ai-integration
            |               (shares ai-workspace-generation)
            |               (also needs STEP 2: registry lives under templates/)
            |
            +---> STEP 5  feat-analyze-and-ship
                            (shares security-audit)

  ANY TIME   STEP 3  feat-verify
                       (no shared capability file, no dependency)
```

Recommended sequence: **1 -> 2 -> 3 -> 4 -> 5**. Step 3 can move anywhere.

### Why step 1 must come first

Three capability files are written by two changes each:

| `openspec/specs/<capability>/spec.md` | written by |
|---|---|
| `project-scaffolding` | change 1, change 2 |
| `ai-workspace-generation` | change 1, change 4 |
| `security-audit` | change 1, change 5 |

Change 1 creates these capabilities. Changes 2, 4, and 5 add requirements to them
with an `## ADDED Requirements` delta rather than a `## MODIFIED Requirements`
delta, because the capability does not exist in `openspec/specs/` until change 1
is archived.

If an extending change is archived before change 1, OpenSpec builds the capability
spec from that change's partial requirement set and change 1's requirements land
afterwards out of context. Apply and archive change 1 first, then the extensions.

Each change's `design.md` records this under `## Migration Plan`, and each
`tasks.md` repeats it under `## Workflow follow-up`.

### Where the dependency between changes is real, not just file ordering

- Change 4 needs change 2 landed: the AI tool registry lives under `templates/`,
  which change 2 creates.
- Change 5's first release is cut after change 1, because releasing while the CI
  quality gate is red would publish with a failing gate.
- Change 3 has no hard dependency. It reuses the audit walkers but does not modify
  any capability another change owns.

---

## What each change is for

### 1. `fix-ci-and-report-truth` — make the tool honest and make CI green

**Problem.** All 16 recorded CI runs failed. The latest run passes 12 of 13 jobs;
only `Backend (laravel)` fails, because `composer install` exits 2: the generated
`laravel/framework ^11.0` constraint resolves only to versions blocked by four
active security advisories. Three further defects: audit reports carry a
hardcoded date (`chrono_placeholder()` returns `"2026-07-28"`), the README
promises `.cursorrules` and `AGENTS.md` that `src/ai_workspace.rs` never writes,
and there is no `--version`.

**Changes.** Laravel constraint to `^12.0` (Laravel 13 needs PHP 8.3; CI pins
8.2), audit reports use the real date via the existing helper in
`src/generator.rs`, the two promised AI files are written, and `version` /
`--version` report the crate version.

**Acceptance.** The `Backend (laravel)` CI job passes — generate, `composer
install`, `php artisan test`, end to end.

### 2. `refactor-templates-and-test-scaffold` — content out of code, tests into projects

**Problem.** `src/stack.rs` is 1449 lines of embedded string literals; changing a
`tsconfig` means editing Rust. And 7 of 9 stacks generate no test file at all,
with no stack declaring a lint or type-check script — while the generated CI
implies there is something to run.

**Changes.** Template content moves to `templates/`, embedded at compile time so
the binary stays self-contained and offline, with a local override directory
(`SNIPE_CODE_TEMPLATES`) that wins per file. Generated projects gain a test entry
point, an example test that exercises generated code, and lint / type-check
commands; the generated CI runs them, gated on the script being declared.

**Acceptance.** All existing tests pass unchanged across the extraction, and CI is
green for every generated-project job.

### 3. `feat-verify` — check a project the tool did not generate

**Problem.** The only verification is `src/bin/verify-generated.rs`, a CI-internal
harness that always generates first, keeps its per-stack command table in a `match`
arm, and cannot be pointed at a project a user already has.

**Changes.** `snipe-code verify [PATH]` derives the applicable gates from the
project's own manifests, runs them, and reports each gate's outcome plus an overall
result and a defined exit status. `--security` adds the ecosystem's dependency
vulnerability scanner; `--only` runs a subset. The CI harness becomes a thin caller
of the same engine, so CI and users exercise one code path.

**Acceptance.** The per-stack CI jobs stay green across the extraction, and
`verify` reports gates matching what each fixture project declares.

### 4. `feat-ai-integration` — artifacts the agent tools actually read

**Problem.** `src/ai_workspace.rs` writes `.ai-workspace/rules.md` and
`.ai-workspace/rules.mdc` — a directory no agent tool reads by default. Change 1
fixes the single documented case; this change makes tool coverage real.

**Changes.** An AI tool selection step (interactive and non-interactive), a
data-driven registry in `templates/ai-tools.json`, and per-tool artifacts at each
tool's documented location — including a `/` command artifact for the tools that
have that concept. Generation can also emit its result as structured data. Nothing
is written outside the project.

**Acceptance.** A generated project works with each selected tool, and the artifact
set matches the registry, with overlapping paths deduplicated.

### 5. `feat-analyze-and-ship` — what is this repo, and how do I install the tool

**Problem.** Nothing reads a codebase and says what it is. And `.github/workflows/
release.yml` has never run — zero tags, zero releases — so the only install path is
"clone it and have Rust". The workflow also asserts its archive count is exactly
`3`, so it breaks the moment a platform is added, and `technical.md` advertises a
macOS ARM build the matrix does not produce. On top of that, `Cargo.toml` declares
none of `description`, `license`, `repository`, `readme`, `keywords`, or
`categories`, so `cargo publish` would be rejected outright and there is no route
onto the ecosystem's package registry. The repository itself also has no
description and no topics, which is why GitHub shows "No description, website, or
topics provided".

**Changes.** `snipe-code analyze [PATH]` reports languages and size, structure and
entry points, and declared dependencies per manifest, read-only and offline, with a
limits section and structured output. Audit findings gain structured output too.
The release target list moves to one file that both the matrix and the publish
assertions read, macOS ARM is added, and `install.sh` / `install.ps1` install
without a toolchain after verifying the published checksum. `Cargo.toml` gains its
publication metadata, and the release also publishes the crate — idempotently,
after the quality gate — so `cargo install` works. The repository gets a
description and topics; the homepage stays unset until a project site exists.
README becomes English with the install path first.

**Acceptance.** A tagged release publishes one archive per documented platform,
each with a checksum and a cosign bundle; the crate version is published and
installable through the package manager; the repository shows a description and
topics; and installing from a release works on a machine with no Rust.

---

## Working through a change

OpenSpec drives the sequence per change:

```
  /openspec-apply-change <change>     implement the tasks
  /openspec-archive-change <change>   merge the capability specs into openspec/specs/
```

Read a change before starting it:

```bash
openspec status   --change <change> --json
openspec show     <change>
openspec validate <change> --strict
```

Local gate before pushing — all three must exit 0:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

Note: this repository has no `cargo` / `rustc` on `PATH` in the current
environment and no `target/` directory, so the gate currently has to run where a
Rust toolchain is installed (or entirely in CI).

---

## Not in this roadmap

Deliberately deferred, so they are tracked rather than forgotten:

- Dockerfile generation for the backend stacks that lack one — only NestJS
  currently gets a Dockerfile and a compose file.
- Template content enrichment — Company Profile generates no files, Portfolio only
  `projects.json`.
- Enriching the generated project README.
- A `list` subcommand for enumerating stacks and templates.
- Widening the audit scan beyond its current file types (no Python, no Ruby).
- A Homebrew tap or any package manager other than the ecosystem's own registry.
- A project website or documentation site; the repository homepage stays unset
  until one exists.

These would form a sixth change.
