# Acceptance matrix

Cases a person runs to accept a release. The `Automation` column names the suite that
covers the same behaviour, so a case marked automated does not need to be repeated by
hand unless the release changed the code behind it.

Status values:

- `automated`: a test in `tests/` asserts it, and the Rust CI job runs that suite.
- `not run`: no executed evidence. The case waits for a human.
- `published`: the artifact exists at the source named in the case. Existence is not the
  same as an install having worked.

| ID | Case | Prerequisite | Expected | Automation | Status |
| --- | --- | --- | --- | --- | --- |
| UAT-01 | Build from source | Rust 1.85 or newer | `cargo build --release` succeeds, `target/release/snipe-code version` prints the crate version | `rust` CI job | automated |
| UAT-02 | First run without install | fresh directory | `./target/release/snipe-code faq` prints the FAQ; a bare `snipe-code` may fail with the "not recognized" message, which the FAQ explains | none | not run |
| UAT-03 | `cargo install snipe-code --locked` | cargo, network | binary lands in `~/.cargo/bin` and resolves on `PATH` | `release_contract_test.rs` asserts the documented flag, not the install itself | not run |
| UAT-04 | Shell installer, Linux | published release | archive downloads, checksum matches, binary runs | `install_script_test.rs` (Linux CI only) | not run |
| UAT-05 | Shell installer, macOS | published release | same as UAT-04 on both architectures | none | not run |
| UAT-06 | PowerShell installer, Windows | published release | archive downloads, checksum matches, `snipe-code.exe` runs | `.github/workflows/release.yml` on `windows-latest` builds the archive only | not run |
| UAT-07 | Container image | Docker | `docker run --rm ghcr.io/athallaarl66/snipe-code:v0.5.3 version` prints `0.5.3` | none | published |
| UAT-08 | Wizard, fully interactive | terminal | each prompt appears in order, project generated, git commit created | none | not run |
| UAT-09 | Generation without a terminal | script | defaults applied (template `none`, frontend `nextjs`), no prompt hangs | `cli_help_test.rs` runs the binary with stdin closed | automated |
| UAT-10 | Generation with every selection flag | none | project generated with the named template and stacks, no prompt shown | `cli_help_test.rs::test_new_accepts_explicit_selection_and_dry_run_alias` | automated |
| UAT-11 | Unknown stack id | none | exit 2, message names the value, no directory created | `cli_help_test.rs` usage-error cases | automated |
| UAT-12 | Empty stack selection | none | exit 2 with the "select at least Frontend or Backend" message | `cli_help_test.rs` | automated |
| UAT-13 | Collision, Cancel | existing directory | nothing is written, the directory is unchanged | `collision_test.rs` | automated |
| UAT-14 | Collision, Overwrite | existing directory | old content replaced, no temporary sibling left behind | `collision_test.rs` | automated |
| UAT-15 | Overwrite that fails mid-publish | existing directory | previous directory restored, recovery path reported | `collision_test.rs` | automated |
| UAT-16 | Dry run | none | structure printed, no file created | `dry_run_test.rs` | automated |
| UAT-17 | `analyze` on a mixed project | none | language totals, structure, entry points, dependencies, Limits section | `analyze_test.rs` | automated |
| UAT-18 | `analyze --json` | none | one JSON object on stdout, nothing else | `analyze_test.rs` | automated |
| UAT-19 | `audit` interactive format prompt | terminal | format menu appears, chosen formats exported | `cli_help_test.rs` covers the non-interactive branch | not run |
| UAT-20 | `audit` without a terminal | script | Markdown exported, default notice printed | `cli_help_test.rs::test_audit_non_interactive_defaults_to_markdown` | automated |
| UAT-21 | `audit --export pdf,docx` | none | both files written into the scanned directory | covered by audit tests through the library API | automated |
| UAT-22 | `audit --json` | none | report files written, exported-files summary on stderr, stdout parses | `structured_output_test.rs` | automated |
| UAT-23 | Audit exit codes | none | `0` clean, `1` HIGH finding, `2` usage error, `3` runtime error | `cli_help_test.rs`, `structured_output_test.rs` | automated |
| UAT-24 | `verify` on a generated project | toolchains installed | gates run in order, each reports passed, failed, not-applicable, or unavailable | `generated-frontend` and `generated-backend` CI jobs | automated |
| UAT-25 | `verify --no-install` | dependencies present | install gate reported not-applicable with the skip reason | `verify_test.rs` | automated |
| UAT-26 | `verify` on an unrecognized project | none | exit 3 with the nothing-executed message | `verify_cli_test.rs` | automated |
| UAT-27 | Opt-in agent artifacts | none | selecting a tool writes only that tool's files plus the shared rules; selecting none writes only the rules | `ai_integration_test.rs` | automated |
| UAT-28 | Template override | override directory | overridden file wins, other files keep the built-in default | `template_override_test.rs` | automated |

## Gaps this matrix records

- No case has been run by a human against the published `v0.5.3` artifacts. Cases UAT-04
  to UAT-07 only confirm the artifacts exist.
- The interactive prompts (UAT-08, UAT-19) have no automated coverage, because a
  terminal-driven run is not exercised in CI.
- Nothing in this table says the audit finds every real vulnerability. The report is
  heuristic by design.
