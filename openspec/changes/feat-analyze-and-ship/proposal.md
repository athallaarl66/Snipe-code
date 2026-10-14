# Proposal

## Why

The tool can build a project and (after `feat-verify`) check one, but it cannot
tell a developer what a project *is*. Nothing reads a codebase and answers the
first question anyone asks about an unfamiliar repository: what languages are here,
how big is each of them, what does this depend on, where does execution start. That
is the capability that makes a tool worth reaching for on a repository it did not
create — and it is cheap, because it is read-only file inspection with the parsing
already partly present (`audit::detect_scopes`, `audit::find_files`).

Distribution is the other half of the same problem. `.github/workflows/release.yml`
has never run: the repository has 26 commits and no tags, so `release.yml` — which
builds three targets, checksums them, and signs them with keyless OIDC — has never
fired. The practical install path is "clone the repository and have a Rust
toolchain", which excludes most of the people the tool is meant for. On top of
that, the documentation has drifted from the workflow: `technical.md` claims a
macOS ARM build and lists `macos-arm64`, while the workflow builds
`x86_64-apple-darwin`. And the workflow's own publish job asserts the archive count
is exactly three, so it fails the moment a platform is added.

This change adds the analysis command and closes the distribution gap so the tool
can actually be installed and seen.

## What Changes

- **`snipe-code analyze [PATH]`** reports what a project is: the languages present
  with their size, the top-level structure and recognized parts, the entry points,
  and the declared dependencies per manifest. Read-only, offline, no new dependency.
- **Analysis states its own limits** — content it cannot recognize is reported as
  unrecognized rather than quietly omitted, so an incomplete report never looks
  complete.
- **Analysis emits structured output** so the report can be consumed by a script or
  agent, matching the human-readable findings.
- **Audit findings gain structured output too**, so the two analysis commands share
  one machine-readable story instead of the audit staying text-only.
- **The release process actually runs**: the first version tag is cut, and the
  platform list is reconciled with the documentation — including a macOS ARM target,
  which the current matrix lacks.
- **The publish count assertion is derived rather than hardcoded**, so adding a
  platform cannot break the release job.
- **Install paths that need no Rust toolchain**: a POSIX shell installer and a
  Windows installer, both verifying the published checksum and signature and both
  failing loudly instead of placing an unverified or partial binary.
- **Repository discovery**: topics and badges so the project is findable, and an
  English README with the install path up front — the repository is public and the
  audience is international.

## Capabilities

### New Capabilities

- `project-analysis`: what the tool reports about an existing project's languages,
  size, structure, entry points, and declared dependencies, and in what form.
- `distribution`: the platform targets built per release, the install paths offered
  to users who have no Rust toolchain, and the integrity information published with
  each release.

### Modified Capabilities

- `security-audit`: audit findings additionally available as structured
  machine-readable data. The delta adds a requirement and leaves the requirements
  introduced by `fix-ci-and-report-truth` intact. **Apply and archive this change
  after `fix-ci-and-report-truth`**, because both write
  `openspec/specs/security-audit/spec.md`.

## Impact

- `src/analyze/` — new module: language and size counting, structure and part
  recognition, entry-point detection, manifest dependency reading, and the two
  report forms.
- `src/audit.rs` — a structured serialization of the existing `Finding` data and
  detected contexts; the rule engine and the existing exporters are unchanged.
- `src/main.rs` — route `analyze` and document it in the help text.
- `src/lib.rs` — expose the new module.
- `.github/workflows/release.yml` — add the macOS ARM target, derive the publish
  archive-count assertion from the built targets, and correct any target naming that
  disagreed with the documentation.
- `install.sh`, `install.ps1` — new installers.
- `README.md` — English rewrite with the install path first, the command list, and
  the supported-platform table; the corrections from `fix-ci-and-report-truth` are
  carried into it rather than reverted.
- Repository settings — topics, and the first tagged release.
- Tests — language and size counting with excluded directories, entry-point and
  manifest detection, the unrecognized-content report, and structured-output
  agreement with the human-readable form.

## Out of scope

Deliberately not included, so they are tracked rather than silently dropped:

- Dockerfile generation for the backend stacks that lack one (only NestJS currently
  gets a Dockerfile and a compose file).
- Enriching the Company Profile and Portfolio templates, which have little or no
  content today, and enriching the generated project README.
- A `list` subcommand for enumerating stacks and templates.
- Widening the audit scan beyond its current file-type set (no Python or Ruby).

## Dependencies

- `fix-ci-and-report-truth` must land and archive first, for the shared
  `security-audit` capability file and because that change makes CI green — a
  release cut while CI is red would publish with a failing quality gate.
- No dependency on the other changes. `analyze` does not reuse the verification
  engine, and the installers operate on published release assets.
