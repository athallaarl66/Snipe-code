# Design

## Context

See `proposal.md` – Why for motivation. What exists and what is known:

- `src/audit.rs` already walks a project: `detect_scopes` recognizes a project and
  returns its scope names, `find_files(path, extension)` and
  `find_dotfiles(path, names)` do the traversal, and `Finding` carries severity and
  location. The exporters (`generate_markdown`, `generate_pdf`, `generate_docx`)
  consume `&[Finding]` and `&[String]` scopes. `run_audit_report` returns an
  `AuditResult` with `exported_files`, `high`, and `policy_failed`.
- `fix-ci-and-report-truth` makes the report date real, adds the `version` command,
  and creates the `security-audit` capability.
- Dependencies available: `serde`, `serde_json`, `dialoguer`, `console`, `printpdf`,
  `docx-rs`. There is no XML parser and no `walkdir`.
- `.github/workflows/release.yml` has three jobs. `quality` runs the local gate.
  `package` builds a three-entry `include` matrix — `x86_64-unknown-linux-gnu`,
  `x86_64-pc-windows-msvc` on `windows-latest`, `x86_64-apple-darwin` on
  `macos-13` — stages `binary` + `LICENSE` + `README.md` + `SOURCE_REVISION` +
  `sha256`, asserts no `.d`/`.rlib` files leaked, and uploads `release-<target>`.
  `publish` downloads all artifacts with `merge-multiple`, then asserts
  `(( ${#archives[@]} == 3 ))` and `(( ${#checksums[@]} == 3 ))`, runs
  `sha256sum --check`, signs each archive with keyless OIDC cosign, and publishes
  with `softprops/action-gh-release` and `generate_release_notes`.
- `technical.md` §11.1 lists `macos-arm64` as a build target and §11.2 names
  `snipe-code-macos-arm64`, while the workflow builds `x86_64-apple-darwin`. The
  documentation and the pipeline disagree.
- The repository is public, MIT, created 2026-07-28, 26 commits, zero tags and zero
  releases, so `release.yml` has never executed. The README is written in
  Indonesian and the repository has no topics or badges.

## Goals / Non-Goals

**Goals:**

- A developer can point the tool at an unfamiliar repository and learn what it is.
- A developer with no Rust toolchain can install the tool and verify the download.
- A tagged release publishes exactly the platforms the documentation claims.
- Analysis and audit both emit machine-readable output.

**Non-Goals:**

- Any build toolchain change: analysis is read-only file inspection.
- An AST, lint, or semantic analysis of the target project.
- A code-versus-comment line split, which would need a per-language parser.
- Publishing to package managers or a Homebrew tap.
- Promotion or discoverability work beyond repository topics and badges.

## Decisions

### Reuse the audit walkers; add only the counting

`find_files` / `find_dotfiles` / `detect_scopes` already traverse a project and are
tested. Analysis adds language mapping and counting on top, rather than a second
walker that can disagree about traversal or exclusions.

### A single extension-to-language table, and a shared exclusion set

Language detection is a data table so adding a language is a row, not a branch.
Traversal skips dependency and build-output directories (`node_modules`, `target`,
`.git`, `dist`, `build`, `vendor`, `bin`, `obj`, `.next`, `.nuxt`, `out`,
`coverage`) so reported size reflects the project rather than its dependencies.

### Report lines, not "lines of code"

The report states file count and total line count, and says plainly that these are
line counts. A code-versus-comment split would require a parser per language, which
is a much larger change and would make the number less trustworthy, not more.

### JSON manifests are parsed; XML manifests are scanned and said to be

`package.json`, `composer.json`, `go.mod`, and `Cargo.toml` are read with `serde_json`
or with a line-oriented scan of the section that declares dependencies. `pom.xml`
and `.csproj` are XML, and there is no XML parser among the dependencies; adding one
for this is not worth it. Those manifests are scanned for dependency element names
and the report states that XML manifests are scanned rather than parsed, which the
"reports its own limits" requirement covers.

*Alternative considered:* shelling out to `dotnet list package` or
`mvn dependency:list` — accurate, but it makes `analyze` depend on installed
toolchains and become slow, contradicting the read-only offline goal.

### `--json` replaces the human report on stdout

For `analyze` there is no progress to separate, so `--json` selects the report form
on stdout. For `audit`, the exported report files are still written and the prose
"exported files" summary moves to stderr, keeping stdout parseable — the same
channel rule that `feat-ai-integration` applies to generation.

### One source of truth for release targets

Move the target list into `.github/release-targets.json`. A job reads it and exposes
it, the `package` matrix consumes it via `fromJSON`, and `publish` derives its
expected archive and checksum counts from the same value. Today the count is
hardcoded as `3`, so the pipeline breaks the moment a platform is added; after this
change, adding a target is one edit in one file.

### Add macOS ARM rather than replacing the Intel macOS target

`technical.md` already advertises a macOS ARM build and the matrix does not produce
one. Add `aarch64-apple-darwin` and keep `x86_64-apple-darwin`: dropping a published
platform would break Intel Mac users, and the cost is one additional macOS runner
job. With one source of truth the added target cannot desynchronise the publish
assertion.

### Installers download, verify, then install — never pipe to a shell

`install.sh` and `install.ps1` resolve the release for the requested version,
download the platform archive to a temporary path, verify the published checksum,
verify the cosign bundle when `cosign` is available, and only then move the binary
onto the install path. Nothing is piped into a shell, a failed check aborts with a
non-zero status, and no partial binary is left behind. A checksum mismatch is a hard
failure, not a warning.

### English README with the Indonesian one preserved

The repository is public and the audience is international, so the primary README
becomes English with the install path first. The existing Indonesian README is kept
as a second file linked from the top rather than deleted, so the current audience is
not dropped. The corrections from `fix-ci-and-report-truth` are carried into the
English text rather than reverted.

## Risks / Trade-offs

- **[Line counts invite misreading as code size]** → The report labels them as line
  counts and states that comments and blanks are included. The alternative is a
  per-language parser, which is out of scope by design.
- **[Scanning XML manifests can over- or under-count dependencies, and a scan is not
  a parse]** → The report states that XML manifests are scanned, and the limits
  section names what was not fully analyzed. Missing a dependency is preferable to
  pulling in an XML parser for one report line.
- **[An installer that downloads and executes is a supply-chain surface]** → Never
  pipe into a shell; download to a temporary path, verify the published checksum
  always and the cosign bundle when available, fail closed, and never place an
  unverified or partial binary. The manual verification steps are documented.
- **[Adding a macOS target increases release wall-clock]** → One extra runner job,
  and the target list is data so it can be trimmed in one edit if it is not worth it.
- **[`analyze` traversal cost on a very large repository]** → Single pass with
  dependency and build directories skipped, matching what the audit already does.
  No caching or parallelism is added; a slow run stays correct.
- **[Cutting the first release while a quality gate is red would publish a broken
  artifact]** → The release depends on `fix-ci-and-report-truth` landing, and
  `release.yml` already gates the package job on the `quality` job.
- **[Topics, badges, and the release itself need repository settings that code cannot
  set]** → Called out as a manual step in the task list rather than assumed done.
- **[Distribution removes install friction but does not create demand]** → Stated
  plainly: this change makes the tool installable and legible, not popular.
  Discoverability beyond topics and badges is out of scope.

## Migration Plan

No data migration. `analyze` is additive; the release changes affect only how
artifacts are produced; the installers are new files.

Ordering and cross-change interaction:

1. `fix-ci-and-report-truth` must land and archive first — it creates
   `openspec/specs/security-audit/spec.md`, which this change appends to, and it
   makes CI green, which the release depends on.
2. Within this change: analysis core → structured output for `analyze` and `audit` →
   single-source release targets plus the macOS ARM target → installers → English
   README and repository topics → cut the first release.
3. The README is edited by more than one change in this set
   (`fix-ci-and-report-truth` corrects claims, this change rewrites it in English).
   Apply them in that order so the corrections are carried into the rewrite, and
   never revert the earlier corrections.

Rollback: revert the commit. A published release is not withdrawn by the revert; if
the first release is defective, delete the tag and re-cut it — that is a deliberate
manual step, not part of the code change.

Version for the first release: `v0.1.0`, matching `Cargo.toml`. Pre-1.0 is honest
for a tool whose command surface is still being completed.

Local pre-push gate: `cargo fmt --check`,
`cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, then a
manual `analyze` run against a multi-language fixture, and a manual install into a
temporary directory from a release asset.

## Open Questions

None. The dependency-manifest handling (parsed versus scanned) and the macOS target
decision are settled above; both are documented limits rather than open choices.
