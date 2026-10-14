# Tasks

## 1. Analysis core

- [ ] 1.1 Add `src/analyze/` with the extension-to-language table and the shared exclusion set, walking the project through the existing audit walkers rather than a second traversal; verify with a test over a multi-language fixture asserting each language's file and line totals
- [ ] 1.2 Verify excluded directories are skipped: add a test fixture containing a `node_modules`-style dependency directory and assert its files appear in no language total
- [ ] 1.3 Implement top-level structure and part recognition, reusing `audit::detect_scopes`, and entry-point detection for the recognized parts; verify with a test asserting the structure and the recognized entry point appear in the report
- [ ] 1.4 Implement manifest dependency reading — `package.json` and `composer.json` parsed, `go.mod` and `Cargo.toml` read from their dependency sections, `pom.xml` and `.csproj` scanned for dependency element names; verify with a test per manifest asserting the declared dependencies are listed
- [ ] 1.5 Implement the limits section, naming content that could not be analyzed and stating that XML manifests are scanned rather than parsed and that figures are line counts; verify with a test asserting an unrecognized-language fixture is reported as not analyzed
- [ ] 1.6 Route `analyze [PATH]` in `src/main.rs`, expose the module from `src/lib.rs`, and document the command in the help text; verify by running the binary against the fixture and checking the human-readable report

## 2. Structured output

- [ ] 2.1 Add structured report output for `analyze` on stdout, containing the same findings as the human-readable form; verify with a test asserting stdout parses as structured data and that the languages, structure, dependencies, and limits agree with the human form
- [ ] 2.2 Add structured output for `audit`, keeping the exported report files intact and moving the exported-files prose to stderr so stdout stays parseable; verify with a test asserting the contexts, findings, and totals agree between the structured output and the exported report
- [ ] 2.3 Document both structured-output options in the help text; verify with a test asserting the help output documents them

## 3. Release targets from one source

- [ ] 3.1 Move the build target list into `.github/release-targets.json`, add `aarch64-apple-darwin`, and have the `package` matrix read it; verify with a dry-run of the workflow expression or a local parse check asserting the matrix expands to the expected four targets
- [ ] 3.2 Replace the hardcoded `(( ${#archives[@]} == 3 ))` and `(( ${#checksums[@]} == 3 ))` assertions in the `publish` job with counts derived from the same target list, so adding a target cannot break the job; verify by adding a target in a scratch copy and confirming the assertion follows
- [ ] 3.3 Reconcile the documented platform list with the built list — the macOS ARM target `technical.md` advertises is now built, and every built target is documented; verify by comparing the documentation table against the target file entry by entry

## 4. Installers

- [ ] 4.1 Write `install.sh`: resolve the release for the requested version, map OS and architecture to a target, download the archive to a temporary path, verify the published checksum, verify the cosign bundle when `cosign` is available, install the binary, and print the PATH hint; verify by installing into a temporary directory from a real release asset
- [ ] 4.2 Write `install.ps1` with the same behaviour for Windows using the zip asset; verify by installing into a temporary directory
- [ ] 4.3 Verify the failure paths: a fetch failure and a checksum mismatch must each exit non-zero, explain the failure, leave no partial binary, and never pipe anything into a shell; verify by forcing each failure in a test run and checking the exit status and the install directory

## 5. Documentation and repository discovery

- [ ] 5.1 Rewrite the README in English with the install path first, carrying over the corrections from `fix-ci-and-report-truth` rather than reverting them, and preserving the existing Indonesian README as a linked second file; verify by following the documented install steps from a clean environment
- [ ] 5.2 Document the `analyze` command, its limits section, and its structured-output option in the README; verify with a test asserting the README documents the command
- [ ] 5.3 Add the supported-platform table to the README, generated from the same target list so it cannot drift; verify by comparing the table against the target file
- [ ] 5.4 Add repository topics and build badges; this is a repository-settings step applied by the owner, not a code change — record the applied topics in the pull request description

## 6. First release

- [ ] 6.1 Confirm every prior change in this set has landed and CI is green, then tag `v0.1.0` matching `Cargo.toml`
- [ ] 6.2 Confirm the release publishes one archive per target, each containing the binary, the license, and the source revision, with a checksum and a cosign bundle for each
- [ ] 6.3 Verify a checksum against a downloaded archive, and verify a cosign bundle against the documented identity
- [ ] 6.4 Install from the published release with `install.sh` into a temporary directory and confirm the installed binary reports the released version

## 7. Integration verification

- [ ] 7.1 Run the full local gate — `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --all-targets --all-features` — and confirm all three exit 0
- [ ] 7.2 Run `analyze` against this repository itself and confirm the reported languages, dependencies, and limits match what the repository actually contains
- [ ] 7.3 Confirm CI is green and record the run URL

## Workflow follow-up

- Confirm `fix-ci-and-report-truth` has been archived before archiving this change — both write `openspec/specs/security-audit/spec.md`.
- Archive, then confirm `openspec/specs/project-analysis/` and `openspec/specs/distribution/` exist and the new requirement is appended under `openspec/specs/security-audit/`.
