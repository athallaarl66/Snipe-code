# Tasks

## 1. Verification option

- [x] 1.1 Add `skip_install: bool` to `verify::Options`, parse `--no-install`, and export the report reason as `verify::SKIP_INSTALL_REASON`
- [x] 1.2 In `verify::run`, rewrite the install gate of every planned part to `not-applicable` with that reason instead of running it
- [x] 1.3 Reject `--no-install` combined with `--only install` as a usage error before any gate runs
- [x] 1.4 Cover the three outcomes with tests: gate skipped while the rest runs, nothing left to execute, contradictory selectors

## 2. CLI surface

- [x] 2.1 Add `--no-install` to `VERIFY_HELP` (usage line, options, example) and to the general `HELP_TEXT`
- [x] 2.2 State in `VERIFY_HELP` that the project's own gates write inside the project, and name an offline alternative
- [x] 2.3 Add FAQ entries for the run directory of each command and for the files `verify` and `audit` create

## 3. Generated project output

- [x] 3.1 Add `bin/`, `obj/`, `Audit_Report.md`, `Audit_Report.pdf`, and `Audit_Report.docx` to `generator::gitignore_content`
- [x] 3.2 Assert the new entries in the generated-`.gitignore` test and keep the single-`vendor/` guard passing

## 4. Documentation

- [x] 4.1 Add the "where each command runs and what it writes" table to `README.md` and `README.id.md`
- [x] 4.2 Document the audit report location and the non-interactive Markdown default in both READMEs
- [x] 4.3 Document the `verify` filesystem effects and the `--no-install` option in both READMEs
- [x] 4.4 List the non-interactive wizard defaults by value and state where `new` writes the project
- [x] 4.5 Add the two risk rows to `docs/features/technical.md`

## 5. Verification

- [x] 5.1 `cargo fmt --check` and `cargo clippy --all-targets --all-features -- -D warnings` both clean
- [x] 5.2 `cargo test --all-targets --all-features` passes (178 tests, 0 failed)
- [x] 5.3 Real run in a throwaway project: `verify <tmp> --no-install` reports `not-applicable install skipped by --no-install` and exits `3`; `verify <tmp> --no-install --only install` exits `2`
- [x] 5.4 Real run of `verify <tmp> --only install` leaves `package-lock.json`, confirming the documented mutation
- [x] 5.5 Real run of `new --name demo --ai-tools none` produces the documented `.gitignore`
- [x] 5.6 Installed binary refreshed with `cargo install --path . --locked`, so `snipe-code help verify` shows the option
- [x] 5.7 Prose check on both READMEs: no bold spans, no curly quotes, no added em dash

## 6. Out of scope

- [ ] 6.1 Publishing the crate to crates.io (needs the `CARGO_REGISTRY_TOKEN` repository secret)
- [ ] 6.2 Re-cutting the `v0.5.0` tag whose release came out empty
