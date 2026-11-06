# Tasks

## 1. Analysis core

- [x] 1.1 Add `src/analyze/` with the extension-to-language table and the shared exclusion set, walking the project through the existing audit walkers rather than a second traversal; verify with a test over a multi-language fixture asserting each language's file and line totals
- [x] 1.2 Verify excluded directories are skipped: add a test fixture containing a `node_modules`-style dependency directory and assert its files appear in no language total
- [x] 1.3 Implement top-level structure and part recognition, reusing `audit::detect_scopes`, and entry-point detection for the recognized parts; verify with a test asserting the structure and the recognized entry point appear in the report
- [x] 1.4 Implement manifest dependency reading — `package.json` and `composer.json` parsed, `go.mod` and `Cargo.toml` read from their dependency sections, `pom.xml` and `.csproj` scanned for dependency element names; verify with a test per manifest asserting the declared dependencies are listed
- [x] 1.5 Implement the limits section, naming content that could not be analyzed and stating that XML manifests are scanned rather than parsed and that figures are line counts; verify with a test asserting an unrecognized-language fixture is reported as not analyzed
- [x] 1.6 Route `analyze [PATH]` in `src/main.rs`, expose the module from `src/lib.rs`, and document the command in the help text; verify by running the binary against the fixture and checking the human-readable report

## 2. Structured output

- [x] 2.1 Add structured report output for `analyze` on stdout, containing the same findings as the human-readable form; verify with a test asserting stdout parses as structured data and that the languages, structure, dependencies, and limits agree with the human form
- [x] 2.2 Add structured output for `audit`, keeping the exported report files intact and moving the exported-files prose to stderr so stdout stays parseable; verify with a test asserting the contexts, findings, and totals agree between the structured output and the exported report
- [x] 2.3 Document both structured-output options in the help text; verify with a test asserting the help output documents them

## 3. Release targets from one source

- [x] 3.1 Move the build target list into `.github/release-targets.json`, add `aarch64-apple-darwin`, and have the `package` matrix read it; verify with a dry-run of the workflow expression or a local parse check asserting the matrix expands to the expected four targets
- [x] 3.2 Replace the hardcoded `(( ${#archives[@]} == 3 ))` and `(( ${#checksums[@]} == 3 ))` assertions in the `publish` job with counts derived from the same target list, so adding a target cannot break the job; verify by adding a target in a scratch copy and confirming the assertion follows
- [x] 3.3 Reconcile the documented platform list with the built list — the macOS ARM target `technical.md` advertises is now built, and every built target is documented; verify by comparing the documentation table against the target file entry by entry

## 4. Installers

- [ ] 4.1 Write `install.sh`: resolve the release for the requested version, map OS and architecture to a target, download the archive to a temporary path, verify the published checksum, verify the cosign bundle when `cosign` is available, install the binary, and print the PATH hint; verify by installing into a temporary directory from a real release asset
- [x] 4.2 Write `install.ps1` with the same behaviour for Windows using the zip asset; verify by installing into a temporary directory
- [x] 4.3 Verify the failure paths: a fetch failure and a checksum mismatch must each exit non-zero, explain the failure, leave no partial binary, and never pipe anything into a shell; verify by forcing each failure in a test run and checking the exit status and the install directory

## 5. Publication metadata and crate publishing

- [x] 5.1 Add the publication metadata to `Cargo.toml` — `description`, `license = "MIT"`, `repository`, `readme`, `keywords`, and `categories` — and confirm the declared license matches the `LICENSE` file that GitHub detects as MIT; verify with `cargo publish --dry-run` completing without a missing-metadata error
- [ ] 5.2 Add an idempotent crate-publish step to the release workflow, ordered after the quality gate, that checks the registry first and skips a version already published; verify by running the step against an already-published version and confirming it reports the existing version without failing the run
- [ ] 5.3 Assert the crate version matches the release tag before publishing, so a mismatch fails the release instead of publishing the wrong version; verify with a deliberately mismatched tag on a scratch branch
- [ ] 5.4 Confirm nothing is published while the quality gate is red, by checking that the publish job's dependency chain includes the quality gate; verify by reading the workflow's job dependencies and by a deliberately failing gate on a scratch branch

## 6. Documentation and repository discovery

- [ ] 6.1 Rewrite the README in English with the install path first, carrying over the corrections from `fix-ci-and-report-truth` rather than reverting them, and preserving the existing Indonesian README as a linked second file; verify by following the documented install steps from a clean environment
- [x] 6.2 Document the `analyze` command, its limits section, and its structured-output option in the README; verify with a test asserting the README documents the command
- [x] 6.3 Add the supported-platform table to the README, generated from the same target list so it cannot drift; verify by comparing the table against the target file
- [x] 6.4 Document both install paths — the shell installers and the ecosystem package manager — and state what each one requires; verify with a test asserting the README documents both paths
- [ ] 6.5 Set the repository description and topics, and leave the homepage unset until a project site exists; this is a repository-settings step applied by the owner, not a code change — record the applied values in the pull request description

## 7. First release

- [ ] 7.1 Confirm every prior change in this set has landed and CI is green, then tag `v0.1.0` matching `Cargo.toml`
- [ ] 7.2 Confirm the release publishes one archive per target, each containing the binary, the license, and the source revision, with a checksum and a cosign bundle for each
- [ ] 7.3 Confirm the crate version is published, and that the registry page shows the description and the MIT license
- [ ] 7.4 Verify a checksum against a downloaded archive, and verify a cosign bundle against the documented identity
- [ ] 7.5 Install from the published release with `install.sh` into a temporary directory and confirm the installed binary reports the released version
- [ ] 7.6 Install through the ecosystem package manager and confirm the installed binary reports the released version

## 8. Integration verification

- [x] 8.1 Run the full local gate — `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-targets --all-features` — and confirm all three exit 0
- [x] 8.2 Run `analyze` against this repository itself and confirm the reported languages, dependencies, and limits match what the repository actually contains
- [ ] 8.3 Confirm CI is green and record the run URL

## Workflow follow-up

- Confirm `fix-ci-and-report-truth` has been archived before archiving this change — both write `openspec/specs/security-audit/spec.md`.
- Archive, then confirm `openspec/specs/project-analysis/` and `openspec/specs/distribution/` exist and the new requirement is appended under `openspec/specs/security-audit/`.

## Status

Completed and verified this session (evidence: `tests/analyze_test.rs`,
`tests/structured_output_test.rs`, `tests/release_contract_test.rs`,
`tests/install_script_test.rs`, plus the manual runs recorded in the pull request):

- 1.1-1.6, 2.1-2.3 — the `analyze` command, its limits section, and structured
  output for both `analyze` and `audit`.
- 3.1-3.3 — `.github/release-targets.json` as the single source of truth, derived
  release validation, and the documentation table reconciled with it.
- 4.2, 4.3 — `install.ps1` installed a real archive into a temporary directory and
  refused a tampered checksum; `install.sh` was verified the same way for the
  success, checksum-mismatch, and fetch-failure paths.
- 5.1 — `cargo publish --dry-run` completes without a missing-metadata error.
- 6.2-6.4 — the English README documents `analyze`, both install paths, and the
  platform table derived from the target list.
- 8.1 — `cargo fmt --check`, Clippy `-D warnings`, and `cargo test` all exit 0.
- 8.2 — `analyze .` against this repository: Rust 40 files / 9356 lines matches an
  independent count, and the reported dependencies match `Cargo.toml`.

Still open, because each needs a push, a tag, or a repository-owner action:

- 4.1, 6.1 — verified against a locally staged release archive served over HTTP;
  the "real release asset" step cannot run before the first release exists (7.1).
- 5.2, 5.3, 5.4 — the workflow logic was proven locally (crates.io answers 200 for
  a published version and 404 for an unpublished one; the tag assertion exits 1 for
  `v9.9.9` and 0 for `v0.1.0`; the `crate` job declares `needs: quality`), but the
  scratch-branch statements need a push.
- 6.5 — repository description and topics are owner-applied settings; the values to
  apply are recorded in the pull request description.
- 7.1-7.6, 8.3 — cutting `v0.1.0`, publishing the crate, and recording the CI run URL
  require the repository owner to push and tag.
